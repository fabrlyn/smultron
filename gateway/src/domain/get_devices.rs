use crate::application::port::GetDevices;

use super::device::Device;

pub async fn get_devices<P>(port: &P) -> Vec<Device>
where
    P: GetDevices + ?Sized,
{
    port.get_devices().await
}
