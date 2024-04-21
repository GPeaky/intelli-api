use ahash::AHashMap;
use tracing::error;

use crate::{protos::packet_header::PacketType, structs::OptionalMessage};

pub struct PacketCaching {
    car_motion: Option<Vec<u8>>,
    session_data: Option<Vec<u8>>,
    participants: Option<Vec<u8>>,
    history_data: AHashMap<u8, Vec<u8>>,
    event_data: AHashMap<[u8; 4], Vec<Vec<u8>>>,
}

// Todo - Create a method that returns a batched packed with all the latest information with a mini cache of 3 seconds
impl PacketCaching {
    pub fn new() -> Self {
        Self {
            car_motion: None,
            session_data: None,
            participants: None,
            history_data: AHashMap::with_capacity(20),
            event_data: AHashMap::with_capacity(5),
        }
    }

    // Todo - This returns all the packets batched
    pub fn get(&self) -> Vec<u8> {
        Vec::new()
    }

    pub fn save(
        &mut self,
        packet_type: PacketType,
        payload: &[u8],
        second_param: Option<OptionalMessage>,
    ) {
        match packet_type {
            PacketType::CarMotion => self.set_car_motion(payload),
            PacketType::SessionData => self.set_session_data(payload),
            PacketType::Participants => self.set_participants(payload),

            PacketType::EventData => {
                debug_assert!(second_param.is_some());

                if let Some(OptionalMessage::Code(code)) = second_param {
                    // Todo - try to avoid .to_vec()
                    self.push_event(payload.to_vec(), code);
                } else {
                    error!("Error Receiving OptionalMessage");
                }
            }

            PacketType::SessionHistoryData => {
                debug_assert!(second_param.is_some());

                if let Some(OptionalMessage::Number(car_id)) = second_param {
                    // Todo - try to avoid .to_vec()
                    self.set_history_data(payload.to_vec(), car_id)
                } else {
                    error!("Error Receiving OptionalMessage");
                }
            }

            PacketType::FinalClassificationData => {
                todo!()
            }
        }
    }

    #[inline(always)]
    fn set_car_motion(&mut self, payload: &[u8]) {
        match &mut self.car_motion {
            Some(car_motion) => {
                car_motion.clear();
                car_motion.extend_from_slice(payload)
            }

            None => {
                let mut car_motion = Vec::with_capacity(payload.len());
                car_motion.extend_from_slice(payload);
                self.car_motion = Some(car_motion);
            }
        }
    }

    #[inline(always)]
    fn set_session_data(&mut self, payload: &[u8]) {
        match &mut self.session_data {
            Some(vec) => {
                vec.clear();
                vec.extend_from_slice(payload);
            }
            None => {
                let mut vec = Vec::with_capacity(payload.len());
                vec.extend_from_slice(payload);
                self.session_data = Some(vec);
            }
        }
    }

    #[inline(always)]
    fn set_participants(&mut self, payload: &[u8]) {
        match &mut self.participants {
            Some(vec) => {
                vec.clear();
                vec.extend_from_slice(payload);
            }
            None => {
                let mut vec = Vec::with_capacity(payload.len());
                vec.extend_from_slice(payload);
                self.participants = Some(vec);
            }
        }
    }

    #[inline(always)]
    fn set_history_data(&mut self, payload: Vec<u8>, car_idx: u8) {
        self.history_data.insert(car_idx, payload);
    }

    #[inline(always)]
    fn push_event(&mut self, payload: Vec<u8>, code: [u8; 4]) {
        self.event_data.entry(code).or_default().push(payload);
    }
}
