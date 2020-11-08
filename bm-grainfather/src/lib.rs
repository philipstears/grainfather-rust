use bm_bluetooth::*;
use std::convert::TryFrom;

pub const SERVICE_ID: u128 = 0x0000cdd000001000800000805f9b34fb;
pub const CHARACTERISTIC_ID_READ: u128 = 0x0003cdd100001000800000805f9b0131;
pub const CHARACTERISTIC_ID_WRITE: u128 = 0x0003cdd200001000800000805f9b0131;

pub type InteractionCode = u8;

#[derive(Debug)]
pub enum Voltage {
    V110,
    V230,
}

#[derive(Debug)]
pub enum Units {
    Fahrenheit,
    Celsius,
}

#[derive(Debug)]
pub enum GrainfatherNotification {
    Temp {
        desired: f64,
        current: f64,
    },
    DelayedHeatTimer {
        active: bool,
        // If zero, the time is inactive, otherwise, it's always the number of remaining minutes +
        // 1, ergo, if it reads 2, there's 1 minute remaining, and possibly some seconds too.
        remaining_minutes: u32,
        remaining_seconds: u32,
        // The total number of minutes remaining + 1
        total_start_time: u32,
    },
    Status1 {
        heat_active: bool,
        pump_active: bool,
        auto_mode_active: bool,
        stage_ramp_active: bool,
        interaction_mode_active: bool,
        interaction_code: InteractionCode,
        stage_number: u8,
        delayed_heat_mode_active: bool,
    },
    Status2 {
        heat_power_output_percentage: u8,
        timer_paused: bool,
        step_mash_mode: bool,
        recipe_interrupted: bool,
        manual_power_mode: bool,
        sparge_water_alert_displayed: bool,
    },
    Interaction {
        interaction_code: InteractionCode,
    },
    Boil {
        boil_temperature: f64,
    },
    VoltageAndUnits {
        voltage: Voltage,
        units: Units,
    },
    FirmwareVersion {
        firmware_version: String,
    },
    Other(char, String),
}

#[derive(Debug)]
pub enum GrainfatherNotificationConvertError {
    InvalidUtf8(std::str::Utf8Error),
}

impl TryFrom<&[u8]> for GrainfatherNotification {
    type Error = GrainfatherNotificationConvertError;

    fn try_from(message: &[u8]) -> Result<Self, Self::Error> {
        let ndata = std::str::from_utf8(message).map_err(Self::Error::InvalidUtf8)?;
        let mut ndata_chars = ndata.chars();
        let ndata_type = ndata_chars.next().unwrap();
        let mut ndata_fields = ndata_chars.as_str().split(",");

        match ndata_type {
            'X' => {
                let desired = ndata_fields.next().unwrap().parse().unwrap();
                let current = ndata_fields.next().unwrap().parse().unwrap();
                Ok(Self::Temp {
                    desired,
                    current,
                })
            }

            'T' => {
                let active = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let remaining_minutes = ndata_fields.next().unwrap().parse().unwrap();
                let total_start_time = ndata_fields.next().unwrap().parse().unwrap();
                let remaining_seconds = ndata_fields.next().unwrap().parse().unwrap();
                Ok(Self::DelayedHeatTimer {
                    active,
                    remaining_minutes,
                    remaining_seconds,
                    total_start_time,
                })
            }

            'Y' => {
                let heat_active = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let pump_active = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let auto_mode_active = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let stage_ramp_active = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let interaction_mode_active = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let interaction_code = ndata_fields.next().unwrap().parse().unwrap();
                let stage_number = ndata_fields.next().unwrap().parse().unwrap();
                let delayed_heat_mode_active = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                Ok(Self::Status1 {
                    heat_active,
                    pump_active,
                    auto_mode_active,
                    stage_ramp_active,
                    interaction_mode_active,
                    interaction_code,
                    stage_number,
                    delayed_heat_mode_active,
                })
            }

            'W' => {
                let heat_power_output_percentage = ndata_fields.next().unwrap().parse().unwrap();
                let timer_paused = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let step_mash_mode = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let recipe_interrupted = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let manual_power_mode = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let sparge_water_alert_displayed = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                Ok(Self::Status2 {
                    heat_power_output_percentage,
                    timer_paused,
                    step_mash_mode,
                    recipe_interrupted,
                    manual_power_mode,
                    sparge_water_alert_displayed,
                })
            }

            'I' => {
                let interaction_code = ndata_fields.next().unwrap().parse().unwrap();
                Ok(Self::Interaction {
                    interaction_code,
                })
            }

            'C' => {
                let boil_temperature = ndata_fields.next().unwrap().parse().unwrap();
                Ok(Self::Boil {
                    boil_temperature,
                })
            }

            'F' => {
                let firmware_version = ndata_fields.next().unwrap().to_string();
                Ok(Self::FirmwareVersion {
                    firmware_version,
                })
            }

            'V' => {
                let voltage_is_110 = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;
                let units_are_celsius = ndata_fields.next().unwrap().parse::<u8>().unwrap() == 1;

                Ok(Self::VoltageAndUnits {
                    voltage: if voltage_is_110 {
                        Voltage::V110
                    } else {
                        Voltage::V230
                    },
                    units: if units_are_celsius {
                        Units::Celsius
                    } else {
                        Units::Fahrenheit
                    },
                })
            }

            _ => Ok(Self::Other(ndata_type, ndata_chars.as_str().to_string())),
        }
    }
}

pub enum Delay {
    Minutes(u32),
    MinutesSeconds(u32, u8),
}

pub enum GrainfatherCommand {
    Reset,
    GetFirmwareVersion,
    GetVoltageAndUnits,
    GetBoilTemperature,

    ToggleHeatActive,
    SetHeatActive(bool),

    TogglePumpActive,
    SetPumpActive(bool),

    // NOTE: minutes is odd, {2, 0} will only run for 1 minute, and {2, 30} will run for 1 minute
    // 30 seconds, {1, 30} and {0, 30} will both run for 30 seconds
    EnableDelayedHeatTimer {
        minutes: u32,
        seconds: u8,
    },

    CancelActiveTimer,

    UpdateActiveTimer(Delay),
    PauseOrResumeActiveTimer,

    IncrementTargetTemperature,
    DecrementTargetTemperature,
    SetTargetTemperature(f64),
    SetLocalBoilTemperature(f64),

    DismissBoilAdditionAlert,
    CancelOrFinishSession,
    PressSet,
    DisableSpargeWaterAlert,
    ResetRecipeInterrupted,

    SetSpargeCounterActive(bool),
    SetBoilControlActive(bool),
    SetManualPowerControlActive(bool),
    SetSpargeAlertModeActive(bool),
}

impl GrainfatherCommand {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut output = String::with_capacity(19);

        match self {
            Self::Reset => {
                output.push('Z');
            }

            Self::GetFirmwareVersion => {
                output.push('X');
            }

            Self::GetVoltageAndUnits => {
                output.push('g');
            }

            Self::GetBoilTemperature => {
                output.push('M');
            }

            Self::ToggleHeatActive => {
                output.push('H');
            }

            Self::SetHeatActive(active) => {
                output.push('K');

                if *active {
                    output.push('1');
                } else {
                    output.push('0');
                }
            }

            Self::TogglePumpActive => {
                output.push('P');
            }

            Self::SetPumpActive(active) => {
                output.push('L');

                if *active {
                    output.push('1');
                } else {
                    output.push('0');
                }
            }

            Self::EnableDelayedHeatTimer {
                minutes,
                seconds,
            } => {
                output.push('B');
                output.push_str(minutes.to_string().as_ref());
                output.push(',');
                output.push_str(seconds.to_string().as_ref());
            }

            Self::CancelActiveTimer => {
                output.push('C');
            }

            Self::UpdateActiveTimer(delay) => match delay {
                Delay::MinutesSeconds(minutes, seconds) => {
                    output.push('W');
                    output.push_str(minutes.to_string().as_ref());
                    output.push(',');
                    output.push_str(seconds.to_string().as_ref());
                }

                Delay::Minutes(minutes) => {
                    output.push('S');
                    output.push_str(minutes.to_string().as_ref());
                }
            },

            Self::PauseOrResumeActiveTimer => {
                output.push('G');
            }

            Self::IncrementTargetTemperature => {
                output.push('U');
            }

            Self::DecrementTargetTemperature => {
                output.push('D');
            }

            Self::SetTargetTemperature(temp) => {
                output.push('$');
                output.push_str(temp.to_string().as_ref());
            }

            Self::SetLocalBoilTemperature(temp) => {
                output.push('E');
                output.push_str(temp.to_string().as_ref());
            }

            Self::DismissBoilAdditionAlert => {
                output.push('A');
            }

            Self::CancelOrFinishSession => {
                output.push('F');
            }

            Self::PressSet => {
                output.push('T');
            }

            Self::DisableSpargeWaterAlert => {
                output.push('V');
            }

            Self::ResetRecipeInterrupted => {
                output.push('!');
            }

            Self::SetSpargeCounterActive(active) => {
                output.push('d');

                if *active {
                    output.push('1');
                } else {
                    output.push('0');
                }
            }

            Self::SetBoilControlActive(active) => {
                output.push('e');

                if *active {
                    output.push('1');
                } else {
                    output.push('0');
                }
            }

            Self::SetManualPowerControlActive(active) => {
                output.push('f');

                if *active {
                    output.push('1');
                } else {
                    output.push('0');
                }
            }

            Self::SetSpargeAlertModeActive(active) => {
                output.push('h');

                if *active {
                    output.push('1');
                } else {
                    output.push('0');
                }
            }
        }

        for _ in 0..(19 - output.len()) {
            output.push(' ');
        }

        output.into()
    }
}

#[derive(Debug)]
pub struct Grainfather {}

#[derive(Debug)]
pub enum GrainfatherConvertError {
    ServiceIdNotFound,
}

impl TryFrom<EIRData<'_>> for Grainfather {
    type Error = GrainfatherConvertError;

    fn try_from(report: EIRData) -> Result<Self, Self::Error> {
        for entry in report.into_iter() {
            if let EIREntry::ServiceIds(ids) = entry {
                if let Some(_) = ids.iter().find(|id| id.as_u128() == SERVICE_ID) {
                    return Ok(Self {});
                }
            }
        }

        Err(Self::Error::ServiceIdNotFound)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
