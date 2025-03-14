//! This library implement the basic [`Tensor`] struct From help of [`DLTensor`].It can accept the data from different device and datatype.

use std::fmt;

use paste::paste;

use persia_libs::{anyhow::Result, half::f16, thiserror, tracing};

#[cfg(feature = "cuda")]
use crate::cuda::GPUStorage;
use crate::dlpack::*;

use persia_speedy::{Readable, Writable};

/// TensorError cover the error of [`Tensor`].
#[derive(Debug, thiserror::Error)]
pub enum TensorError {
    #[error("cpu storagea not found")]
    CPUStorageNotFound,
    #[error("gpu storage not found")]
    GPUStorageNotFound,
}

/// Enum representation of rust datatype.
#[derive(Readable, Writable, Copy, Clone, Debug)]
pub enum DType {
    F16 = 1,
    F32 = 2,
    F64 = 3,
    I8 = 4,
    I16 = 5,
    I32 = 6,
    I64 = 7,
    U8 = 8,
    U16 = 9,
    U32 = 10,
    U64 = 11,
    USIZE = 12,
}

impl fmt::Display for DType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl DType {
    /// Bit size of current datatype.
    pub fn get_type_size(&self) -> usize {
        match self {
            DType::F16 => std::mem::size_of::<f16>(),
            DType::F32 => std::mem::size_of::<f32>(),
            DType::F64 => std::mem::size_of::<f64>(),
            DType::I8 => std::mem::size_of::<i8>(),
            DType::I16 => std::mem::size_of::<i16>(),
            DType::I32 => std::mem::size_of::<i32>(),
            DType::I64 => std::mem::size_of::<i64>(),
            DType::U8 => std::mem::size_of::<u8>(),
            DType::U16 => std::mem::size_of::<u16>(),
            DType::U32 => std::mem::size_of::<u32>(),
            DType::U64 => std::mem::size_of::<u64>(),
            DType::USIZE => std::mem::size_of::<usize>(),
        }
    }

    /// Name of current datatype
    pub fn get_type_name(&self) -> String {
        self.to_string()
    }

    /// Convert to [`DLDataType`].
    pub fn to_dldtype(&self) -> DLDataType {
        let (code, bits) = match self {
            DType::F16 => (*&DLDataTypeCode::DLFloat, 16),
            DType::F32 => (*&DLDataTypeCode::DLFloat, 32),
            DType::F64 => (*&DLDataTypeCode::DLFloat, 64),
            DType::I8 => (*&DLDataTypeCode::DLInt, 8),
            DType::I16 => (*&DLDataTypeCode::DLInt, 16),
            DType::I32 => (*&DLDataTypeCode::DLInt, 32),
            DType::I64 => (*&DLDataTypeCode::DLInt, 64),
            DType::U8 => (*&DLDataTypeCode::DLUInt, 8),
            DType::U16 => (*&DLDataTypeCode::DLUInt, 16),
            DType::U32 => (*&DLDataTypeCode::DLUInt, 32),
            DType::U64 => (*&DLDataTypeCode::DLUInt, 64),
            DType::USIZE => (*&DLDataTypeCode::DLUInt, 64),
        };
        let code = code as u8;

        DLDataType {
            code,
            bits,
            lanes: 1,
        }
    }
}

/// Storarge that store the vector data.
#[derive(Readable, Writable, Debug)]
pub enum CPUStorage {
    F16(Vec<f16>),
    F32(Vec<f32>),
    F64(Vec<f64>),
    I32(Vec<i32>),
    I64(Vec<i64>),
    U32(Vec<u32>),
    U64(Vec<u64>),
    USIZE(Vec<usize>),
}

impl CPUStorage {
    pub fn get_dtype(&self) -> DType {
        match self {
            CPUStorage::F16(_) => DType::F16,
            CPUStorage::F32(_) => DType::F32,
            CPUStorage::F64(_) => DType::F64,
            CPUStorage::I32(_) => DType::I32,
            CPUStorage::I64(_) => DType::I64,
            CPUStorage::U32(_) => DType::U32,
            CPUStorage::U64(_) => DType::U64,
            CPUStorage::USIZE(_) => DType::USIZE,
        }
    }

    pub fn get_raw_ptr(&mut self) -> *mut std::os::raw::c_void {
        match self {
            CPUStorage::F32(val) => val.as_ptr() as *mut std::os::raw::c_void,
            CPUStorage::F16(val) => val.as_ptr() as *mut std::os::raw::c_void,
            CPUStorage::F64(val) => val.as_ptr() as *mut std::os::raw::c_void,
            CPUStorage::I32(val) => val.as_ptr() as *mut std::os::raw::c_void,
            CPUStorage::I64(val) => val.as_ptr() as *mut std::os::raw::c_void,
            CPUStorage::U32(val) => val.as_ptr() as *mut std::os::raw::c_void,
            CPUStorage::U64(val) => val.as_ptr() as *mut std::os::raw::c_void,
            CPUStorage::USIZE(val) => val.as_ptr() as *mut std::os::raw::c_void,
        }
    }

    pub fn data_ptr(&mut self) -> u64 {
        self.get_raw_ptr() as u64
    }
}

macro_rules! add_new_func2_cpu_storage {
    ($(($typ:ty, $attr:ident)),*) => {
        paste! {
            impl CPUStorage {
                    $(
                        pub fn [<from_ $typ:lower>](data: Vec<$typ>) -> Self {
                            CPUStorage::$attr(data)
                        }
                    )*
            }
        }
    }
}

add_new_func2_cpu_storage!(
    (f16, F16),
    (f32, F32),
    (f64, F64),
    (i32, I32),
    (i64, I64),
    (usize, USIZE),
    (u32, U32),
    (u64, U64)
);

#[derive(Readable, Writable, Debug)]
pub enum Storage {
    CPU(CPUStorage),

    #[cfg(feature = "cuda")]
    GPU(GPUStorage),
}

impl Storage {
    pub fn consume_cpu_storage(self) -> Result<CPUStorage, TensorError> {
        match self {
            Storage::CPU(val) => Ok(val),
            _ => Err(TensorError::CPUStorageNotFound),
        }
    }
}

#[derive(Readable, Writable, Debug)]
pub enum DeviceType {
    CPU,
    GPU,
}

#[derive(Readable, Writable, Debug)]
pub struct Device {
    device_type: DeviceType,
    device_id: Option<i32>,
}

impl Device {
    fn with_device_id(device_id: Option<i32>) -> Self {
        match &device_id {
            Some(device_id) => Device {
                device_type: DeviceType::GPU,
                device_id: Some(*device_id),
            },
            None => Device {
                device_type: DeviceType::CPU,
                device_id: None,
            },
        }
    }

    fn to_dldevicetype(&self) -> DLDevice {
        match self.device_type {
            DeviceType::CPU => DLDevice {
                device_id: 0i32,
                device_type: DLDeviceType::DLCPU,
            },
            DeviceType::GPU => DLDevice {
                device_id: *self.device_id.as_ref().unwrap(),
                device_type: DLDeviceType::DLCUDA,
            },
        }
    }
}

impl Default for Device {
    fn default() -> Self {
        Device {
            device_type: DeviceType::CPU,
            device_id: None,
        }
    }
}

pub fn get_stride_by_shape(shape: &[usize]) -> Vec<i64> {
    let dim = shape.len();
    let mut result = vec![1i64; dim];

    for i in 1..dim {
        result[dim - i - 1] = result[dim - i] * shape[dim - i] as i64;
    }
    result
}

#[derive(Readable, Writable, Debug)]
pub struct Tensor {
    pub storage: Storage,
    pub shape: Vec<usize>,
    pub stride: Vec<i64>,
    pub name: Option<String>,
    pub device: Device,
}

impl Tensor {
    pub fn new(
        storage: Storage,
        shape: Vec<usize>,
        name: Option<String>,
        device_id: Option<i32>,
    ) -> Self {
        let stride = get_stride_by_shape(shape.as_slice());
        let device = Device::with_device_id(device_id);

        Self {
            storage,
            shape,
            stride,
            name,
            device,
        }
    }

    #[cfg(feature = "cuda")]
    pub fn to(self, device: &Option<i32>) -> Tensor {
        if let Some(device_id) = device {
            self.cuda(*device_id)
        } else {
            self
        }
    }

    #[cfg(not(feature = "cuda"))]
    pub fn to(self, _device: &Option<i32>) -> Tensor {
        self
    }

    #[cfg(feature = "cuda")]
    pub fn cuda(self, device_id: i32) -> Tensor {
        let shape = self.shape.clone();
        let cpu_storage = self.storage.consume_cpu_storage().unwrap();
        let gpu_storage = GPUStorage::new(cpu_storage, shape).unwrap();

        Tensor {
            storage: Storage::GPU(gpu_storage),
            shape: self.shape,
            stride: self.stride,
            name: self.name,
            device: Device::with_device_id(Some(device_id)),
        }
    }

    pub fn device(&self) -> String {
        match self.storage {
            Storage::CPU(_) => "cpu".to_owned(),
            #[cfg(feature = "cuda")]
            Storage::GPU(_) => "cuda".to_owned(),
        }
    }

    pub fn raw_data_ptr(&mut self) -> *mut std::os::raw::c_void {
        match &mut self.storage {
            Storage::CPU(val) => val.get_raw_ptr(),
            #[cfg(feature = "cuda")]
            Storage::GPU(val) => val.get_raw_ptr(),
        }
    }

    pub fn data_ptr(&mut self) -> u64 {
        self.raw_data_ptr() as u64
    }

    pub fn dtype(&self) -> DType {
        match &self.storage {
            Storage::CPU(val) => val.get_dtype(),
            #[cfg(feature = "cuda")]
            Storage::GPU(val) => val.get_dtype(),
        }
    }

    pub fn dlpack(&mut self) -> DLManagedTensor {
        let dl_tensor = DLTensor {
            data: self.raw_data_ptr(),
            device: self.device.to_dldevicetype(),
            ndim: self.shape.len() as i32,
            dtype: self.dtype().to_dldtype(),
            shape: self.shape.as_mut_ptr() as *mut i64,
            strides: self.stride.as_mut_ptr(),
            byte_offset: 0u64,
        };
        tracing::debug!(
            "dltensor device dtype is {:?}, shape is {:?}, strides is {:?}",
            &dl_tensor.device,
            &self.shape,
            &self.stride
        );
        DLManagedTensor {
            dl_tensor,
            manager_ctx: std::ptr::null_mut(),
            deleter: Some(drop_dl_managed_tensor),
        }
    }
}

#[derive(Readable, Writable, Debug)]
pub struct SparseTensor {
    pub data: Storage,
    pub offset: Vec<u64>,
    pub name: Option<String>,
}
