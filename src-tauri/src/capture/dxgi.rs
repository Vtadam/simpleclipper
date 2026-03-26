//! DXGI Desktop Duplication capture implementation.
//!
//! Uses D3D11 and DXGI to capture the full primary display at high frame rates.
//! Frames are returned as raw BGRA byte slices.

use anyhow::{anyhow, Result};
use std::time::Instant;

pub struct CapturedFrame {
    pub data: Vec<u8>, // BGRA, width * height * 4 bytes
    pub width: u32,
    pub height: u32,
    pub pts_ms: i64,
}

#[cfg(target_os = "windows")]
mod windows_impl {
    use super::*;
    use windows::{
        core::Interface,
        Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE,
        Win32::Graphics::Direct3D11::{
            D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D,
            D3D11_CPU_ACCESS_READ, D3D11_MAPPED_SUBRESOURCE, D3D11_MAP_READ,
            D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING, D3D11_SDK_VERSION,
        },
        Win32::Graphics::Dxgi::{
            IDXGIAdapter, IDXGIDevice, IDXGIOutput, IDXGIOutput1, IDXGIOutputDuplication,
            IDXGIResource, DXGI_ERROR_ACCESS_LOST, DXGI_ERROR_WAIT_TIMEOUT,
            DXGI_OUTDUPL_FRAME_INFO,
        },
    };

    pub struct DxgiCapture {
        device: ID3D11Device,
        context: ID3D11DeviceContext,
        duplication: IDXGIOutputDuplication,
        width: u32,
        height: u32,
        start: Instant,
    }

    impl DxgiCapture {
        pub fn new() -> Result<Self> {
            // SAFETY: D3D11CreateDevice is a well-defined Windows API call.
            // We pass null for adapter (use default) and null for the feature levels array
            // to get the highest supported level. The out-pointers are valid stack locations.
            let (device, context) = unsafe {
                let mut device: Option<ID3D11Device> = None;
                let mut context: Option<ID3D11DeviceContext> = None;
                D3D11CreateDevice(
                    None,
                    D3D_DRIVER_TYPE_HARDWARE,
                    None,
                    Default::default(),
                    None,
                    D3D11_SDK_VERSION,
                    Some(&mut device),
                    None,
                    Some(&mut context),
                )?;
                (
                    device.ok_or_else(|| anyhow!("D3D11 device is null"))?,
                    context.ok_or_else(|| anyhow!("D3D11 context is null"))?,
                )
            };

            // SAFETY: QueryInterface calls are safe when the source COM object is valid
            // and the requested interface is implemented by that object.
            let (duplication, width, height) = unsafe {
                let dxgi_device: IDXGIDevice = device.cast()?;
                let adapter: IDXGIAdapter = dxgi_device.GetAdapter()?;
                // Use output index 0 (primary monitor)
                let output: IDXGIOutput = adapter.EnumOutputs(0)?;
                let desc = output.GetDesc()?;
                let output1: IDXGIOutput1 = output.cast()?;
                let width = (desc.DesktopCoordinates.right - desc.DesktopCoordinates.left) as u32;
                let height = (desc.DesktopCoordinates.bottom - desc.DesktopCoordinates.top) as u32;

                let duplication = output1.DuplicateOutput(&device)?;
                (duplication, width, height)
            };

            Ok(Self {
                device,
                context,
                duplication,
                width,
                height,
                start: Instant::now(),
            })
        }

        pub fn capture_frame(&mut self) -> Result<Option<CapturedFrame>> {
            // SAFETY: AcquireNextFrame is a DXGI API that returns a new frame.
            // We properly release the frame after mapping and copying the data.
            unsafe {
                let mut frame_info = DXGI_OUTDUPL_FRAME_INFO::default();
                let mut desktop_resource: Option<IDXGIResource> = None;

                // Timeout of 33ms (~30fps). Returns DXGI_ERROR_WAIT_TIMEOUT if no new frame.
                let hr = self.duplication.AcquireNextFrame(
                    33,
                    &mut frame_info,
                    &mut desktop_resource,
                );

                if let Err(ref e) = hr {
                    if e.code() == DXGI_ERROR_WAIT_TIMEOUT {
                        return Ok(None);
                    }
                    if e.code() == DXGI_ERROR_ACCESS_LOST {
                        // Display mode changed or desktop was locked — recreate duplication
                        self.recreate_duplication()?;
                        return Ok(None);
                    }
                }
                hr?;

                let resource = desktop_resource.ok_or_else(|| anyhow!("null desktop resource"))?;
                let frame_texture: ID3D11Texture2D = resource.cast()?;

                // Create a staging texture with CPU read access
                let mut desc = D3D11_TEXTURE2D_DESC::default();
                frame_texture.GetDesc(&mut desc);
                desc.Usage = D3D11_USAGE_STAGING;
                desc.BindFlags = Default::default();
                desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ.0 as u32;
                desc.MiscFlags = Default::default();

                let mut staging_texture: Option<ID3D11Texture2D> = None;
                self.device
                    .CreateTexture2D(&desc, None, Some(&mut staging_texture))?;
                let staging = staging_texture.ok_or_else(|| anyhow!("null staging texture"))?;

                // Copy from GPU frame to staging texture so we can map it
                // SAFETY: Both textures are valid D3D11 resources on the same device.
                self.context.CopyResource(&staging, &frame_texture);

                // Map the staging texture to get CPU-accessible pointer
                let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
                self.context
                    .Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))?;

                let width = self.width as usize;
                let height = self.height as usize;
                let row_pitch = mapped.RowPitch as usize;
                let mut data = Vec::with_capacity(width * height * 4);

                // Copy row by row to handle potential stride padding
                // SAFETY: `mapped.pData` is valid while the texture is mapped.
                // We read exactly `height` rows of `width * 4` bytes each.
                let src_ptr = mapped.pData as *const u8;
                for row in 0..height {
                    let src_row = std::slice::from_raw_parts(
                        src_ptr.add(row * row_pitch),
                        width * 4,
                    );
                    data.extend_from_slice(src_row);
                }

                self.context.Unmap(&staging, 0);

                // Always release the frame after we're done reading
                self.duplication.ReleaseFrame()?;

                let pts_ms = self.start.elapsed().as_millis() as i64;

                Ok(Some(CapturedFrame {
                    data,
                    width: self.width,
                    height: self.height,
                    pts_ms,
                }))
            }
        }

        pub fn width(&self) -> u32 {
            self.width
        }

        pub fn height(&self) -> u32 {
            self.height
        }

        /// Recreate the output duplication after access lost (e.g., lock screen, mode change).
        fn recreate_duplication(&mut self) -> Result<()> {
            // SAFETY: Same as in new() — valid COM object casts and API calls.
            unsafe {
                let dxgi_device: IDXGIDevice = self.device.cast()?;
                let adapter: IDXGIAdapter = dxgi_device.GetAdapter()?;
                let output: IDXGIOutput = adapter.EnumOutputs(0)?;
                let output1: IDXGIOutput1 = output.cast()?;
                self.duplication = output1.DuplicateOutput(&self.device)?;
            }
            Ok(())
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod stub_impl {
    use super::*;

    pub struct DxgiCapture {
        width: u32,
        height: u32,
        start: Instant,
    }

    impl DxgiCapture {
        pub fn new() -> Result<Self> {
            Ok(Self { width: 1920, height: 1080, start: Instant::now() })
        }

        pub fn capture_frame(&mut self) -> Result<Option<CapturedFrame>> {
            let w = self.width as usize;
            let h = self.height as usize;
            let data = vec![128u8; w * h * 4];
            Ok(Some(CapturedFrame {
                data,
                width: self.width,
                height: self.height,
                pts_ms: self.start.elapsed().as_millis() as i64,
            }))
        }

        pub fn width(&self) -> u32 { self.width }
        pub fn height(&self) -> u32 { self.height }
    }
}

#[cfg(target_os = "windows")]
pub use windows_impl::DxgiCapture;

#[cfg(not(target_os = "windows"))]
pub use stub_impl::DxgiCapture;
