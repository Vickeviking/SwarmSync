use crate::core::PulseSubscriptions;
use crate::services::{ServiceChannels, ServiceWiring};
use crate::utils::Logger;

use std::sync::Arc;
use tokio::sync::Mutex;

pub struct SharedResources {
    pub logger: Arc<Logger>,
    pub pulse_subscriptions: Arc<PulseSubscriptions>,
    pub service_channels: Arc<ServiceChannels>, //only has ref ones so only Arc
    pub service_wiring: Arc<Mutex<ServiceWiring>>, // builds on take() so needs write = MutexLock
}

impl SharedResources {
    pub fn new(
        logger: Arc<Logger>,
        pulse_subscriptions: Arc<PulseSubscriptions>,
        service_channels: Arc<ServiceChannels>,
        service_wiring: Arc<Mutex<ServiceWiring>>,
    ) -> Self {
        SharedResources {
            logger,
            pulse_subscriptions,
            service_channels,
            service_wiring,
        }
    }

    pub fn get_logger(&self) -> Arc<Logger> {
        Arc::clone(&self.logger)
    }

    pub fn get_pulse_subscriptions(&self) -> Arc<PulseSubscriptions> {
        Arc::clone(&self.pulse_subscriptions)
    }

    pub fn get_service_channels(&self) -> Arc<ServiceChannels> {
        Arc::clone(&self.service_channels)
    }

    pub fn get_service_wiring(&self) -> Arc<Mutex<ServiceWiring>> {
        //we dont lock, they lock we minimize cruitical section
        Arc::clone(&self.service_wiring)
    }
}
