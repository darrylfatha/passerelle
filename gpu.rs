// Dipakai oleh aplikasi dan installer (lewat include!).
//
// Menggambar lewat DirectX 12 (atau Vulkan), bukan OpenGL. Di laptop dengan dua GPU
// (mis. AMD terintegrasi + NVIDIA) dan layar refresh tinggi, penukaran buffer OpenGL bisa
// sesaat menampilkan buffer kosong sehingga seluruh jendela berkedip setiap kali kursor
// bergerak. GPU terintegrasi (yang tersambung ke layar laptop) dipilih agar frame tidak perlu
// disalin antar-GPU. OpenGL hanya cadangan terakhir untuk PC yang sangat lama.

pub fn wgpu_options() -> eframe::egui_wgpu::WgpuConfiguration {
    use eframe::egui_wgpu::{WgpuConfiguration, WgpuSetup};
    use eframe::wgpu;
    use std::sync::Arc;

    let mut config = WgpuConfiguration::default();
    if let WgpuSetup::CreateNew(setup) = &mut config.wgpu_setup {
        setup.instance_descriptor.backends = wgpu::Backends::DX12 | wgpu::Backends::VULKAN | wgpu::Backends::GL;
        setup.power_preference = wgpu::PowerPreference::LowPower;
        setup.native_adapter_selector = Some(Arc::new(|adapters, surface| {
            let rank = |a: &wgpu::Adapter| {
                let info = a.get_info();
                let backend = match info.backend {
                    wgpu::Backend::Dx12 => 0,
                    wgpu::Backend::Vulkan => 1,
                    wgpu::Backend::Gl => 2,
                    _ => 3,
                };
                let device = match info.device_type {
                    wgpu::DeviceType::IntegratedGpu => 0,
                    wgpu::DeviceType::DiscreteGpu => 1,
                    wgpu::DeviceType::VirtualGpu | wgpu::DeviceType::Other => 2,
                    wgpu::DeviceType::Cpu => 3,
                };
                (backend, device)
            };
            adapters
                .iter()
                .filter(|a| surface.map_or(true, |s| a.is_surface_supported(s)))
                .min_by_key(|a| rank(a))
                .cloned()
                .ok_or_else(|| "Tidak ada kartu grafis yang bisa menampilkan jendela ini.".to_string())
        }));
    }
    config
}
