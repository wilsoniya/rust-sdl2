use sys::joystick as ll;

use JoystickSubsystem;
use SdlResult;
use get_error;
use clear_error;
use sys::event::{SDL_QUERY, SDL_ENABLE};
use std::ffi::{CString, CStr, NulError};
use std::fmt::{Display, Formatter, Error};
use libc::c_char;

impl JoystickSubsystem {
    /// Retreive the total number of attached joysticks *and* controllers identified by SDL.
    pub fn num_joysticks(&self) -> SdlResult<u32> {
        let result = unsafe { ll::SDL_NumJoysticks() };

        if result >= 0 {
            Ok(result as u32)
        } else {
            Err(get_error())
        }
    }

    /// Attempt to open the joystick at number `id` and return it.
    pub fn open(&self, id: u32) -> SdlResult<Joystick> {
        let id = try!(u32_to_int!(id));

        let joystick = unsafe { ll::SDL_JoystickOpen(id) };

        if joystick.is_null() {
            Err(get_error())
        } else {
            Ok(Joystick {
                subsystem: self.clone(),
                raw: joystick
            })
        }
    }

    /// Return the name of the joystick at index `id`
    pub fn name_for_index(&self, id: u32) -> SdlResult<String> {
        let id = try!(u32_to_int!(id));
        let name = unsafe { ll::SDL_JoystickNameForIndex(id) };

        c_str_to_string_or_err(name)
    }

    /// Get the GUID for the joystick number `id`
    pub fn device_guid(&self, id: u32) -> SdlResult<Guid> {
        let id = try!(u32_to_int!(id));

        let raw = unsafe { ll::SDL_JoystickGetDeviceGUID(id) };

        let guid = Guid { raw: raw };

        if guid.is_zero() {
            Err(get_error())
        } else {
            Ok(guid)
        }
    }

    /// If state is `true` joystick events are processed, otherwise
    /// they're ignored.
    pub fn set_event_state(&self, state: bool) {
        unsafe { ll::SDL_JoystickEventState(state as i32) };
    }

    /// Return `true` if joystick events are processed.
    pub fn event_state(&self) -> bool {
        unsafe { ll::SDL_JoystickEventState(SDL_QUERY as i32)
                 == SDL_ENABLE as i32 }
    }

    /// Force joystick update when not using the event loop
    #[inline]
    pub fn update(&self) {
        unsafe { ll::SDL_JoystickUpdate() };
    }

}

/// Wrapper around the SDL_Joystick object
pub struct Joystick {
    subsystem: JoystickSubsystem,
    raw: *mut ll::SDL_Joystick
}

impl Joystick {
    #[inline]
    pub fn subsystem(&self) -> &JoystickSubsystem { &self.subsystem }

    /// Return the name of the joystick or an empty string if no name
    /// is found.
    pub fn name(&self) -> String {
        let name = unsafe { ll::SDL_JoystickName(self.raw) };

        c_str_to_string(name)
    }

    /// Return true if the joystick has been opened and currently
    /// connected.
    pub fn attached(&self) -> bool {
        unsafe { ll::SDL_JoystickGetAttached(self.raw) != 0 }
    }

    pub fn instance_id(&self) -> i32 {
        let result = unsafe { ll::SDL_JoystickInstanceID(self.raw) };

        if result < 0 {
            // Should only fail if the joystick is NULL.
            panic!(get_error())
        } else {
            result
        }
    }

    /// Retreive the joystick's GUID
    pub fn guid(&self) -> Guid {
        let raw = unsafe { ll::SDL_JoystickGetGUID(self.raw) };

        let guid = Guid { raw: raw };

        if guid.is_zero() {
            // Should only fail if the joystick is NULL.
            panic!(get_error())
        } else {
            guid
        }
    }

    /// Retreive the number of axes for this joystick
    pub fn num_axes(&self) -> u32 {
        let result = unsafe { ll::SDL_JoystickNumAxes(self.raw) };

        if result < 0 {
            // Should only fail if the joystick is NULL.
            panic!(get_error())
        } else {
            result as u32
        }
    }

    /// Gets the position of the given `axis`.
    ///
    /// The function will fail if the joystick doesn't have the provided axis.
    pub fn axis(&self, axis: u32) -> SdlResult<i16> {
        // This interface is a bit messed up: 0 is a valid position
        // but can also mean that an error occured. As far as I can
        // tell the only way to know if an error happened is to see if
        // get_error() returns a non-empty string.
        clear_error();

        let axis = try!(u32_to_int!(axis));
        let pos = unsafe { ll::SDL_JoystickGetAxis(self.raw, axis) };

        if pos != 0 {
            Ok(pos)
        } else {
            let err = get_error();

            if err.0.is_empty() {
                Ok(pos)
            } else {
                Err(err)
            }
        }
    }

    /// Retreive the number of buttons for this joystick
    pub fn num_buttons(&self) -> u32 {
        let result = unsafe { ll::SDL_JoystickNumButtons(self.raw) };

        if result < 0 {
            // Should only fail if the joystick is NULL.
            panic!(get_error())
        } else {
            result as u32
        }
    }

    /// Return `Ok(true)` if `button` is pressed.
    ///
    /// The function will fail if the joystick doesn't have the provided button.
    pub fn button(&self, button: u32) -> SdlResult<bool> {
        // Same deal as axis, 0 can mean both unpressed or
        // error...
        clear_error();

        let button = try!(u32_to_int!(button));
        let pressed = unsafe { ll::SDL_JoystickGetButton(self.raw, button) };

        match pressed {
            1 => Ok(true),
            0 => {
                let err = get_error();

                if err.0.is_empty() {
                    // Button is not pressed
                    Ok(false)
                } else {
                    Err(err)
                }
            }
            // Should be unreachable
            _ => Err(get_error()),
        }
    }

    /// Retreive the number of balls for this joystick
    pub fn num_balls(&self) -> u32 {
        let result = unsafe { ll::SDL_JoystickNumBalls(self.raw) };

        if result < 0 {
            // Should only fail if the joystick is NULL.
            panic!(get_error())
        } else {
            result as u32
        }
    }

    /// Return a pair `(dx, dy)` containing the difference in axis
    /// position since the last poll
    pub fn ball(&self, ball: u32) -> SdlResult<(i32, i32)> {
        let mut dx = 0;
        let mut dy = 0;

        let ball = try!(u32_to_int!(ball));
        let result = unsafe { ll::SDL_JoystickGetBall(self.raw, ball, &mut dx, &mut dy) };

        if result == 0 {
            Ok((dx, dy))
        } else {
            Err(get_error())
        }
    }

    /// Retreive the number of balls for this joystick
    pub fn num_hats(&self) -> u32 {
        let result = unsafe { ll::SDL_JoystickNumHats(self.raw) };

        if result < 0 {
            // Should only fail if the joystick is NULL.
            panic!(get_error())
        } else {
            result as u32
        }
    }

    /// Return the position of `hat` for this joystick
    pub fn hat(&self, hat: u32) -> SdlResult<HatState> {
        // Guess what? This function as well uses 0 to report an error
        // but 0 is also a valid value (HatState::Centered). So we
        // have to use the same hack as `axis`...
        clear_error();

        let hat = try!(u32_to_int!(hat));
        let result = unsafe { ll::SDL_JoystickGetHat(self.raw, hat) };

        let state = HatState::from_raw(result as u8);

        if result != 0 {
            Ok(state)
        } else {
            let err = get_error();

            if err.0.is_empty() {
                Ok(state)
            } else {
                Err(err)
            }
        }
    }
}

impl Drop for Joystick {
    fn drop(&mut self) {
        if self.attached() {
            unsafe { ll::SDL_JoystickClose(self.raw) }
        }
    }
}

/// Wrapper around a SDL_JoystickGUID, a globally unique identifier
/// for a joystick.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Guid {
    raw: ll::SDL_JoystickGUID,
}

impl Guid {
    /// Create a GUID from a string representation.
    pub fn from_string(guid: &str) -> Result<Guid, NulError> {
        let guid = try!(CString::new(guid));

        let raw = unsafe { ll::SDL_JoystickGetGUIDFromString(guid.as_ptr()) };

        Ok(Guid { raw: raw })
    }

    /// Return `true` if GUID is full 0s
    pub fn is_zero(&self) -> bool {
        for &i in self.raw.data.iter() {
            if i != 0 {
                return false;
            }
        }

        return true;
    }

    /// Return a String representation of GUID
    pub fn string(&self) -> String {
        // Doc says "buf should supply at least 33bytes". I took that
        // to mean that 33bytes should be enough in all cases, but
        // maybe I'm wrong?
        let mut buf = [0; 33];

        let len   = buf.len() as i32;
        let c_str = buf.as_mut_ptr();

        unsafe {
            ll::SDL_JoystickGetGUIDString(self.raw, c_str, len);
        }

        // The buffer should always be NUL terminated (the
        // documentation doesn't explicitely say it but I checked the
        // code)
        c_str_to_string(c_str)
    }

    /// Return a copy of the internal SDL_JoystickGUID
    pub fn raw(self) -> ll::SDL_JoystickGUID {
        self.raw
    }
}

impl Display for Guid {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.string())
    }
}

/// This is represented in SDL2 as a bitfield but obviously not all
/// combinations make sense: 5 for instance would mean up and down at
/// the same time... To simplify things I turn it into an enum which
/// is how the SDL2 docs present it anyway (using macros).
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum HatState {
    Centered  = 0,
    Up        = 0x01,
    Right     = 0x02,
    Down      = 0x04,
    Left      = 0x08,
    RightUp   = 0x02 | 0x01,
    RightDown = 0x02 | 0x04,
    LeftUp    = 0x08 | 0x01,
    Leftdown  = 0x08 | 0x04,
}

impl HatState {
    pub fn from_raw(raw: u8) -> HatState {
        match raw {
            0  => HatState::Centered,
            1  => HatState::Up,
            2  => HatState::Right,
            4  => HatState::Down,
            8  => HatState::Left,
            3  => HatState::RightUp,
            6  => HatState::RightDown,
            9  => HatState::LeftUp,
            12 => HatState::Leftdown,
            _  => panic!("Unexpected hat position: {}", raw),
        }
    }
}

/// Convert C string `c_str` to a String. Return an empty string if
/// c_str is NULL.
fn c_str_to_string(c_str: *const c_char) -> String {
    if c_str.is_null() {
        String::new()
    } else {
        let bytes = unsafe { CStr::from_ptr(c_str).to_bytes() };

        String::from_utf8_lossy(bytes).to_string()
    }
}

/// Convert C string `c_str` to a String. Return an SDL error if
/// `c_str` is NULL.
fn c_str_to_string_or_err(c_str: *const c_char) -> SdlResult<String> {
    if c_str.is_null() {
        Err(get_error())
    } else {
        let bytes = unsafe { CStr::from_ptr(c_str).to_bytes() };

        Ok(String::from_utf8_lossy(bytes).to_string())
    }
}
