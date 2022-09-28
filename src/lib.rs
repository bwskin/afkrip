use std::{thread::sleep, time::Duration};
use rand::Rng;

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
            AxisEventEmitError,
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
                .emit(&[Self::key_event(key, 1)])
                .or(Err(errors::MouseError::ButtonEventEmitError))?;
            Ok(())
        }

        pub fn key_up(&mut self, key: Key) -> Result<(), errors::MouseError> {
            self.device
                .emit(&[Self::key_event(key, 0)])
                .or(Err(errors::MouseError::ButtonEventEmitError))?;
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
                    Self::move_event(RelativeAxisType::REL_Y, y),
                ])
                .or(Err(errors::MouseError::AxisEventEmitError))?;
            Ok(())
        }
    }
}

pub mod cli {
    use clap::{builder::ValueParser, Arg, ArgAction, Command};

    mod validators {
        pub fn idle_time(value: &str) -> Result<u64, &'static str> {
            match value.parse::<u64>() {
                Ok(number) => Ok(number),
                Err(_) => Err("must be valid positive integer"),
            }
        }
        pub fn mouse_range(value: &str) -> Result<i32, &'static str> {
            match value.parse::<i32>() {
                Ok(number) => {
                    if number >= 0 {
                        Ok(number)
                    } else {
                        Err("must be positive integer")
                    }
                }
                Err(_) => Err("must be valid positive integer"),
            }
        }
    }

    pub struct Args {
        pub idle_time: u64,
        pub mouse_range: i32,
    }

    pub fn parse_args() -> Args {
        let cli = Command::new("afkrip")
            .version("0.1.0")
            .author("Naoto <nnomura98@gmail.com>")
            .arg(
                Arg::new("idle_time")
                    .help("Idle time in minutes")
                    .short('i')
                    .long("idle-time")
                    .action(ArgAction::Set)
                    .default_value("10")
                    .value_parser(ValueParser::new(validators::idle_time)),
            )
            .arg(
                Arg::new("mouse_range")
                    .help("Maximum pixels value to move mouse in one axis")
                    .short('m')
                    .long("mouse-range")
                    .action(ArgAction::Set)
                    .default_value("50")
                    .value_parser(ValueParser::new(validators::mouse_range)),
            )
            .get_matches();

        let idle_time: u64 = *cli
            .get_one("idle_time")
            .expect("Validated and have default value");

        let mouse_range: i32 = *cli
            .get_one("mouse_range")
            .expect("Validated and have default value");

        Args {
            idle_time,
            mouse_range
        }
    }
}

fn error_exit(err: input::errors::MouseError) -> ! {
    error(err);
    std::process::exit(1)
}

fn error(error: input::errors::MouseError) -> () {
    println!("{}", error.message());
}

pub fn start() {
    let cli::Args { idle_time, mouse_range } = cli::parse_args();

    println!("Idle time set to: {} minutes", idle_time);
    println!("When idle mouse will be moved by {}px in both axes", mouse_range);

    let mut mouse = match input::Mouse::new() {
        Ok(instance) => instance,
        Err(error) => error_exit(error),
    };

    loop {
        sleep(Duration::from_secs(1));

        let idle = rs_idle::get_idle_time();
        let mut rng = rand::thread_rng();
        if idle > idle_time * 60 * 1000 {
            let x: i32 = rng.gen_range(-mouse_range..mouse_range);
            let y: i32 = rng.gen_range(-mouse_range..mouse_range);
            println!("Idle: {}ms, moving mouse by x: {}, y: {}", idle, x, y);
            mouse.pointer_move(x, y).unwrap_or_else(error);
        }
    }
}
