use std::{error::Error, fmt::Display, sync::{Once, Mutex, PoisonError, MutexGuard}, collections::HashMap, ffi::CStr};

use libcwiid_sys::{cwiid_wiimote_t, cwiid_set_err, wiimote};
use lazy_static::lazy_static;

use crate::WiiMote;

type MutexError<'a> = PoisonError<MutexGuard<'a,HashMap<usize, String>>>;
pub type CwiidResult<'a, T> = Result<T, CwiidError<'a>>;

#[derive(Debug)]
pub enum CwiidError<'a> {
    Internal(InternalError),
    Mutex(MutexError<'a>)
}

impl Display for CwiidError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            CwiidError::Internal(err) => write!(f, "CwiidError[{}]", err),
            CwiidError::Mutex(err) => write!(f, "CwiidError[{}]", err)
        }
    }
}

impl Error for CwiidError<'_> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let CwiidError::Internal(err) = self {
            return Some(err);
        }
        None
    }
}

impl From<InternalError> for CwiidError<'_> {
    fn from(err: InternalError) -> Self {
        Self::Internal(err)
    }
}

impl<'a> From<MutexError<'a>> for CwiidError<'a> {
    fn from(err: MutexError<'a>) -> Self {
        CwiidError::Mutex(err)
    }
}

#[derive(Debug)]
pub struct InternalError(pub i32, pub Option<String>);

impl Display for InternalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.1 {
            Some(msg) =>  write!(f, "InternalError[code:{}, msg:{}]", self.0, msg),
            None => write!(f, "InternalError[code:{}]", self.0)
        }
       
    }
}

impl Error for InternalError {}

static SYNC_OBJ: Once = Once::new();

pub(crate) fn init_error() {
    SYNC_OBJ.call_once(|| {
        unsafe {
            cwiid_set_err(Some(cwiid_error_handler));
        }
    })
}

pub(crate) fn get_error_message<'a>(handle: *const wiimote) -> CwiidResult<'a, Option<String>> {
    let raw_handle = handle as usize;
    let msg = ERROR_MAP.lock()?.remove(&raw_handle);
    Ok(msg)
}

pub(crate) unsafe fn convert_error<'a>(wii_mote: &WiiMote, e: i32) -> CwiidResult<'a, ()> {
    match e {
        0 => Ok(()),
        e => {
            let message = get_error_message(wii_mote.handle)?;
            Err(InternalError(e, message).into())
        }
    }
}

lazy_static!{
    static ref ERROR_MAP: Mutex<HashMap<usize, String>> = Mutex::new(HashMap::new());
}

unsafe extern "C" fn cwiid_error_handler(wiimote: *mut cwiid_wiimote_t, msg: *const i8, _ap: *mut libcwiid_sys::__va_list_tag) {
    let err_msg = CStr::from_ptr(msg).to_str().expect("Couldn't parse error message.").to_owned();
    ERROR_MAP.lock().expect("Couldn't lock on ERROR_MAP.").insert(wiimote as usize, err_msg);
}