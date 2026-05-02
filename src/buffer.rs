use sdl3::{
    Error,
    gpu::{
        Buffer, BufferBinding, BufferMemMap, BufferRegion, BufferUsageFlags, CopyPass, Device,
        TransferBuffer, TransferBufferLocation, TransferBufferUsage,
    },
};

pub struct BufferHelper<T> {
    pub data: Vec<T>,
    buffer: Buffer,
    transfer: TransferBuffer,
}

impl<T: Copy> BufferHelper<T> {
    fn data_size(data: &Vec<T>) -> u32 {
        (data.len() * size_of::<T>()) as u32
    }

    pub fn new(device: &Device, data: Vec<T>, max_size: u32) -> Result<Self, Error> {
        let data_size = Self::data_size(&data);
        let out = Self {
            data,
            buffer: device
                .create_buffer()
                .with_usage(BufferUsageFlags::VERTEX)
                .with_size(size_of::<T>() as u32 * max_size)
                .build()?,
            transfer: device
                .create_transfer_buffer()
                .with_usage(TransferBufferUsage::UPLOAD)
                .with_size(data_size)
                .build()?,
        };
        out.refresh_transfer(device);

        Ok(out)
    }

    pub fn refresh_transfer(&self, device: &Device) {
        let mut map: BufferMemMap<'_, T> = self.transfer.map(&device, false);
        map.mem_mut().copy_from_slice(&self.data);
        map.unmap();
    }

    pub fn upload_transfer(&self, copy: &CopyPass) {
        copy.upload_to_gpu_buffer(
            TransferBufferLocation::default().with_transfer_buffer(&self.transfer),
            BufferRegion::default()
                .with_buffer(&self.buffer)
                .with_size(Self::data_size(&self.data)),
            false,
        );
    }

    pub fn binding(&self) -> BufferBinding {
        BufferBinding::default()
            .with_buffer(&self.buffer)
            .with_offset(0)
    }
}
