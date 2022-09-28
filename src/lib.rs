pub mod input {
    use evdev::{
        uinput::VirtualDevice, uinput::VirtualDeviceBuilder, AttributeSet, BusType, EventType,
        InputEvent, InputId, Key, RelativeAxisType,
    };
    use std::{thread::sleep, time::Duration};

    pub mod errors {
        pub enum MouseError {
            BuilderInitError,
            KeysAssignError,
            AxesAssignError,
            CreateDeviceError,
            ButtonEventEmitError,
            AxisEventEmitError
        }

        impl MouseError {
            pub fn message(&self) -> &'static str {
                match self {
                    Self::BuilderInitError => "Cannot initialize device builder",
                    Self::KeysAssignError => "Cannot assign keys to virtual mouse",
                    Self::AxesAssignError => "Cannot assign axes to virtual mouse",
                    Self::CreateDeviceError => "Cannot create virtual mouse",
                    Self::AxisEventEmitError => "Cannot move virtual mouse",
                    Self::ButtonEventEmitError => "Cannot click virtual mouse",
                }
            }
        }
    }

    pub struct Mouse {
        device: VirtualDevice,
    }

    impl Mouse {
        pub fn new() -> Result<Self, errors::MouseError> {
            let mut keys: AttributeSet<Key> = AttributeSet::new();
            keys.insert(Key::BTN_LEFT);
            keys.insert(Key::BTN_RIGHT);

            let mut axes: AttributeSet<RelativeAxisType> = AttributeSet::new();
            axes.insert(RelativeAxisType::REL_X);
            axes.insert(RelativeAxisType::REL_Y);

            let mouse = VirtualDeviceBuilder::new()
                .or(Err(errors::MouseError::BuilderInitError))?
                .name("KPRS mouse device")
                .with_keys(&keys)
                .or(Err(errors::MouseError::KeysAssignError))?
                .with_relative_axes(&axes)
                .or(Err(errors::MouseError::AxesAssignError))?
                .input_id(InputId::new(BusType::BUS_USB, 0x0001, 0x0001, 0x0001))
                .build()
                .or(Err(errors::MouseError::CreateDeviceError))?;
            sleep(Duration::from_millis(150));

            Ok(Mouse { device: mouse })
        }

        fn move_event(axis: RelativeAxisType, value: i32) -> InputEvent {
            InputEvent::new(EventType::RELATIVE, axis.0, value)
        }

        fn key_event(key: Key, value: i32) -> InputEvent {
            InputEvent::new(EventType::KEY, key.code(), value)
        }

        pub fn key_down(&mut self, key: Key) -> Result<(), errors::MouseError> {
            self.device
                .emit(&[Self::key_event(key, 1)]).or(Err(errors::MouseError::ButtonEventEmitError))?;
            Ok(())
        }

        pub fn key_up(&mut self, key: Key) -> Result<(), errors::MouseError> {
            self.device
                .emit(&[Self::key_event(key, 0)]).or(Err(errors::MouseError::ButtonEventEmitError))?;
            Ok(())
        }

        pub fn click(&mut self, key: Key) -> Result<(), errors::MouseError> {
            self.key_down(key)?;
            sleep(Duration::from_millis(50));
            self.key_up(key)?;
            Ok(())
        }

        pub fn left_click(&mut self) -> Result<(), errors::MouseError> {
            self.click(Key::BTN_LEFT)?;
            Ok(())
        }

        pub fn right_click(&mut self) -> Result<(), errors::MouseError> {
            self.click(Key::BTN_RIGHT)?;
            Ok(())
        }

        pub fn pointer_move(&mut self, x: i32, y: i32) -> Result<(), errors::MouseError> {
            self.device
                .emit(&[
                      Self::move_event(RelativeAxisType::REL_X, x),
                      Self::move_event(RelativeAxisType::REL_Y, y)
                ]).or(Err(errors::MouseError::AxisEventEmitError))?;
            Ok(())
        }
    }
}
