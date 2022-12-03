use std::fmt::Display;

use bluetooth::Address;
use error::{convert_error, get_error_message, init_error, CwiidResult, InternalError};
use libcwiid_sys::{
    bdaddr_t, cwiid_close, cwiid_disable, cwiid_enable, cwiid_get_state, cwiid_open, cwiid_set_led,
    cwiid_set_rpt_mode, cwiid_set_rumble, cwiid_state, cwiid_wiimote_t, wiimote, CWIID_BATTERY_MAX,
    CWIID_FLAG_MOTIONPLUS, cwiid_get_id,
};
use state_enums::{cb::Button, Acceleration, Bitmap, Feature, Led, IRSensor};

mod bluetooth;
mod error;
mod result_extensions;

#[derive(Debug)]
pub struct WiiMote<'a> {
    handle: &'a mut cwiid_wiimote_t,
}

impl<'a> WiiMote<'a> {
    pub fn new() -> CwiidResult<'a, Self> {
        Self::find(&mut Address::default(), 0)
    }

    pub fn find(address: &mut Address, flags: i32) -> CwiidResult<'a, Self> {
        init_error();
        let handle = Self::open_cwiid(address.as_mut(), flags)?;
        //Self::init_data(handle)?;
        //Self::init_callback(handle)?;
        Ok(Self { handle })
    }

    pub fn set_led(&mut self, led: Led, state: bool) -> CwiidResult<'a, ()> {
        let mut led_state = self.get_raw_state()?.led;
        led.set(&mut led_state, state);
        unsafe {
            let e = cwiid_set_led(self.handle, led_state);
            convert_error(self, e)
        }
    }

    pub fn is_led_on(&mut self, led: Led) -> CwiidResult<'a, bool> {
        let state = led.get(&self.get_raw_state()?.led);
        Ok(state)
    }

    pub fn set_rumble(&mut self, state: bool) -> CwiidResult<'a, ()> {
        unsafe {
            let e = cwiid_set_rumble(self.handle, state as u8);
            convert_error(self, e)
        }
    }

    pub fn is_rumble_enabled(&mut self) -> CwiidResult<'a, bool> {
        let state = self.get_raw_state()?.rumble;
        Ok(state != 0)
    }

    pub fn set_feature(&mut self, feature: Feature, state: bool) -> CwiidResult<'a, ()> {
        let mut feature_state = self.get_raw_state()?.rpt_mode;
        unsafe {
            feature.set(&mut feature_state, state);
            let e = cwiid_set_rpt_mode(self.handle, feature_state);
            convert_error(self, e)
        }
    }

    pub fn is_feature_enabled(&mut self, feature: Feature) -> CwiidResult<'a, bool> {
        let state = feature.get(&self.get_raw_state()?.rpt_mode);
        Ok(state)
    }

    pub fn set_motion_plus(&mut self, state: bool) -> CwiidResult<'a, ()> {
        unsafe {
            let e = if state {
                cwiid_enable(self.handle, CWIID_FLAG_MOTIONPLUS as i32)
            } else {
                cwiid_disable(self.handle, CWIID_FLAG_MOTIONPLUS as i32)
            };
            convert_error(self, e)
        }
    }

    pub fn get_state(&mut self) -> CwiidResult<'a, WiiMoteState> {
        let id = unsafe {
            let id = cwiid_get_id(self.handle);
            convert_error(self, 0)?;
            id
        };
        Ok(WiiMoteState {
            state_data: self.get_raw_state()?,
            id
        })
    }

    fn open_cwiid(address: &mut bdaddr_t, flags: i32) -> CwiidResult<'a, &'a mut wiimote> {
        unsafe {
            let handle = cwiid_open(address, flags);
            match handle.as_mut() {
                Some(handle) => Ok(handle),
                None => {
                    let message = get_error_message(handle)?;
                    Err(InternalError(0, message).into())
                }
            }
        }
    }

    fn get_raw_state(&mut self) -> CwiidResult<'a, cwiid_state> {
        //try making getter functions const in raw C library
        unsafe {
            let mut state_data = std::mem::zeroed();
            let e = cwiid_get_state(self.handle, &mut state_data);
            convert_error(self, e)?;
            Ok(state_data)
        }
    }

    // fn init_data(handle:&mut wiimote) -> CwiidResult<'a, ()> {
    //     check_error(handle)?;
    //     unsafe {
    //         let state_data: Box<cwiid_state> = Box::new(std::mem::zeroed());
    //         let raw_state_data = std::boxed::Box::into_raw(state_data);
    //         let state_data = raw_state_data as *const c_void;
    //         let err = cwiid_set_data(handle, state_data);
    //         if err != 0 {
    //             drop(Box::from_raw(raw_state_data));
    //             cwiid_set_data(handle, std::ptr::null());
    //             return Err("Allocation Error".to_owned().into());
    //         }
    //     }
    //     Ok(())
    // }

    // fn init_callback(handle:&mut wiimote) -> CwiidResult<'a, ()> {
    //     check_error(handle)?;
    //     unsafe {
    //         let err = cwiid_set_mesg_callback(handle, Some(Self::callback));
    //         if err != 0 {
    //             return Err("Set Callback Error".to_owned().into());
    //         }
    //     }
    //     Ok(())
    // }

    // unsafe extern "C" fn callback(handle: *mut wiimote, mesg_cout: i32, mesg: *mut cwiid_mesg, timestamp: *mut timespec) {
    //     let message_slice = std::ptr::slice_from_raw_parts_mut(mesg, mesg_cout as usize);
    //     let state_data = cwiid_get_data(handle) as *mut cwiid_state;
    //     cwiid_get_state(wiimote, state)
    //     for message in &mut *message_slice {
    //         match message.type_ {
    //             cwiid_mesg_type_CWIID_MESG_STATUS => {
    //                 state_data.battery =
    //             }
    //         }
    //     }
    // }
}

impl Drop for WiiMote<'_> {
    fn drop(&mut self) {
        // unsafe {
        //     let state_data = cwiid_get_data(self.handle);
        //     if state_data.is_null() {
        //         return;
        //     }
        //     let raw_state_data = state_data as *mut cwiid_state;
        //     drop(Box::from_raw(raw_state_data));
        //     cwiid_set_data(self.handle, std::ptr::null());
        // }
        unsafe {
            let e = cwiid_close(self.handle);
            convert_error(self, e).unwrap();
        }
    }
}

pub struct WiiMoteState {
    state_data: cwiid_state,
    id: i32
}

impl WiiMoteState {
    pub fn acceleration(&self) -> Acceleration {
        Acceleration::new(self.state_data.acc)
    }

    pub fn button(&self) -> Vec<&Button> {
        let buttons = &self.state_data.buttons;
        Button::VARIANTS.iter().filter(|b| b.get(buttons)).collect()
    }

    pub fn battery(&self) -> f32 {
        (100 * self.state_data.battery) as f32 / CWIID_BATTERY_MAX as f32
    }

    pub fn ir_sensor(&self) -> IRSensor {
        IRSensor::new(&self.state_data.ir_src)
    }
}

impl Display for WiiMoteState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WiiMote: {} <{}%>[{}, {:?}, {}]", self.id, self.battery(), self.acceleration(), self.button(), self.ir_sensor())
    }
}

pub mod state_enums {

    pub use cb as controller_buttons;
    use std::{ops::BitAnd, fmt::Display};

    use libcwiid_sys::{
        cwiid_ir_src, CWIID_LED1_ON, CWIID_LED2_ON, CWIID_LED3_ON, CWIID_LED4_ON, CWIID_RPT_ACC,
        CWIID_RPT_BALANCE, CWIID_RPT_BTN, CWIID_RPT_CLASSIC, CWIID_RPT_IR, CWIID_RPT_MOTIONPLUS,
        CWIID_RPT_NUNCHUK, CWIID_RPT_STATUS, CWIID_X, CWIID_Y, CWIID_Z,
    };
    use num_traits::{Num, One};

    #[derive(Debug, Clone, Copy)]
    pub enum Led {
        Led1 = CWIID_LED1_ON as isize,
        Led2 = CWIID_LED2_ON as isize,
        Led3 = CWIID_LED3_ON as isize,
        Led4 = CWIID_LED4_ON as isize,
    }

    impl Bitmap<u8> for Led {
        fn ordinal(&self) -> u8 {
            *self as u8
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Feature {
        Status = CWIID_RPT_STATUS as isize,
        Button = CWIID_RPT_BTN as isize,
        Acceleration = CWIID_RPT_ACC as isize,
        IR = CWIID_RPT_IR as isize,
        Nunchuk = CWIID_RPT_NUNCHUK as isize,
        Classic = CWIID_RPT_CLASSIC as isize,
        Balance = CWIID_RPT_BALANCE as isize,
        MotionPlus = CWIID_RPT_MOTIONPLUS as isize,
    }

    impl Bitmap<u8> for Feature {
        fn ordinal(&self) -> u8 {
            *self as u8
        }
    }
    pub mod cb {

        use libcwiid_sys::{
            CWIID_BTN_1, CWIID_BTN_2, CWIID_BTN_A, CWIID_BTN_B, CWIID_BTN_DOWN, CWIID_BTN_HOME,
            CWIID_BTN_LEFT, CWIID_BTN_MINUS, CWIID_BTN_PLUS, CWIID_BTN_RIGHT, CWIID_BTN_UP,
        };
        use Button::*;

        use super::Bitmap;

        #[derive(Debug, Clone, Copy)]
        pub enum Button {
            Num2 = CWIID_BTN_2 as isize,
            Num1 = CWIID_BTN_1 as isize,
            B = CWIID_BTN_B as isize,
            A = CWIID_BTN_A as isize,
            Minus = CWIID_BTN_MINUS as isize,
            Home = CWIID_BTN_HOME as isize,
            Left = CWIID_BTN_LEFT as isize,
            Right = CWIID_BTN_RIGHT as isize,
            Down = CWIID_BTN_DOWN as isize,
            Up = CWIID_BTN_UP as isize,
            Plus = CWIID_BTN_PLUS as isize,
        }

        impl Button {
            pub const VARIANTS: &[Button] =
                &[Num2, Num1, B, A, Minus, Home, Left, Right, Down, Up, Plus];
        }

        impl Bitmap<u16> for Button {
            fn ordinal(&self) -> u16 {
                *self as u16
            }
        }
    }

    #[derive(Debug)]
    pub struct Acceleration {
        pub x: u8,
        pub y: u8,
        pub z: u8,
    }

    impl Acceleration {
        pub(crate) fn new(array: [u8; 3]) -> Self {
            Self {
                x: array[CWIID_X as usize],
                y: array[CWIID_Y as usize],
                z: array[CWIID_Z as usize],
            }
        }
    }

    impl Display for Acceleration {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Acceleration[x:{}, y:{}, z:{}]", self.x, self.y, self.z)
    }
    }

    pub struct IRSensor {
        sources: Vec<IRSource>,
    }

    impl IRSensor {
        pub fn new(sources: &[cwiid_ir_src]) -> Self {
            let sources: Vec<IRSource> = sources
                .iter()
                .filter(|src| src.valid != 0)
                .map(|src| IRSource {
                    x: src.pos[CWIID_X as usize],
                    y: src.pos[CWIID_Y as usize],
                    size: src.size,
                })
                .collect();
            Self { sources }
        }

        pub fn sources(&self) -> &[IRSource] {
            self.sources.as_slice()
        }
    }

    impl Display for IRSensor {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut sources = String::new();
            self.sources.iter().for_each(|s| {
                sources.push_str("\n\t");
                sources.push_str(&s.to_string())
            
        });
            write!(f, "IRSensor[{}\n]",sources)
    }
    }

    #[derive(Debug)]
    pub struct IRSource {
        pub x: u16,
        pub y: u16,
        pub size: i8,
    }

    impl Display for IRSource {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "IRSource[x:{}, y:{}, size:{}", self.x, self.y, self.size)
    }
    }

    pub(crate) trait Bitmap<T>
    where
        T: Num
            + std::ops::BitOrAssign
            + std::ops::Not
            + std::ops::BitAndAssign<<T as std::ops::Not>::Output>
            + std::ops::BitAnd
            + Copy,
        <T as BitAnd<T>>::Output: Num,
    {
        fn ordinal(&self) -> T;

        fn set(&self, bitmap: &mut T, state: bool) {
            let mask = self.ordinal();
            if state {
                *bitmap |= mask;
            } else {
                *bitmap &= !mask;
            }
        }

        fn get(&self, bitmap: &T) -> bool {
            let mask = self.ordinal();
            (*bitmap & mask).is_one()
        }
    }
}
