#[cfg(target_os = "ios")]
use crate::native::ios;

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    DownloadFailed,
    AndroidAssetLoadingError,
    /// MainBundle pathForResource returned null
    IOSAssetNoSuchFile,
    /// NSData dataWithContentsOfFile or data.bytes are null
    IOSAssetNoData,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error: {:?}", self)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IOError(e)
    }
}

pub type Response = Result<Vec<u8>, Error>;

/// Filesystem path on desktops
pub fn load_file<F: Fn(Response) + 'static>(path: &str, on_loaded: F) {
    #[cfg(target_os = "android")]
    load_file_android(path, on_loaded);

    #[cfg(target_os = "ios")]
    ios::load_file(path, on_loaded);

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    load_file_desktop(path, on_loaded);
}

#[cfg(target_os = "android")]
fn load_file_android<F: Fn(Response)>(path: &str, on_loaded: F) {
    fn load_file_sync(path: &str) -> Response {
        use crate::native;

        let filename = std::ffi::CString::new(path).unwrap();

        let mut data: native::android_asset = unsafe { std::mem::zeroed() };

        unsafe { native::android::load_asset(filename.as_ptr(), &mut data as _) };

        if !data.content.is_null() {
            let slice =
                unsafe { std::slice::from_raw_parts(data.content, data.content_length as _) };
            let response = slice.iter().map(|c| *c as _).collect::<Vec<_>>();
            Ok(response)
        } else {
            Err(Error::AndroidAssetLoadingError)
        }
    }

    let response = load_file_sync(path);

    on_loaded(response);
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn load_file_desktop<F: Fn(Response)>(path: &str, on_loaded: F) {
    fn load_file_sync(path: &str) -> Response {
        use std::fs::File;
        use std::io::Read;

        let mut response = vec![];
        let mut file = File::open(path)?;
        file.read_to_end(&mut response)?;
        Ok(response)
    }

    let response = load_file_sync(path);

    on_loaded(response);
}
