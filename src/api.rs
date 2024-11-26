use reqwest::Url;
use serde::de::Error as _;
use serde::{de::DeserializeOwned, Deserialize, Deserializer};
use uuid::Uuid;

use crate::domain::DeviceName;
use crate::simulation;

#[derive(Deserialize, Debug, Clone)]
pub struct SimulationOverview {
    pub id: Uuid,
    #[serde(deserialize_with = "from_unix_timestamp")]
    pub timestamp: jiff::Timestamp,
}

fn from_unix_timestamp<'de, D>(de: D) -> Result<jiff::Timestamp, D::Error>
where
    D: Deserializer<'de>,
{
    let n = i64::deserialize(de)?;
    jiff::Timestamp::from_millisecond(n).map_err(D::Error::custom)
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Report {
    pub result: i64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Simulation {
    pub devices: Vec<DeviceName>,
    pub res: Report,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SubmittedSimulation {
    pub id: Uuid,
    pub report: Report,
}

pub struct Client {
    pub endpoint: Url,
    pub client: reqwest::Client,
}

impl Client {
    fn with_path(&self, path: &str) -> Url {
        let mut ret = self.endpoint.clone();
        ret.set_path(path);
        ret
    }

    async fn get_json<T>(&self, path: &str) -> Result<T, reqwest::Error>
    where
        T: DeserializeOwned,
    {
        self.client
            .get(self.with_path(path))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    async fn get_string(&self, path: &str) -> Result<String, reqwest::Error> {
        self.client
            .get(self.with_path(path))
            .send()
            .await?
            .error_for_status()?
            .text()
            .await
    }

    pub async fn fetch_all_simulations(&self) -> Result<Vec<SimulationOverview>, reqwest::Error> {
        self.get_json("/simulation").await
    }

    pub async fn fetch_simulation(&self, id: Uuid) -> Result<Simulation, reqwest::Error> {
        self.get_json(&format!("/simulation/{id}")).await
    }

    pub async fn fetch_simulation_source(&self, id: Uuid) -> Result<String, reqwest::Error> {
        self.get_string(&format!("/simulation/{id}/source")).await
    }

    pub async fn fetch_simulation_log(&self, id: Uuid) -> Result<String, reqwest::Error> {
        self.get_string(&format!("/simulation/{id}/log")).await
    }

    pub async fn fetch_simulation_log_by_device(
        &self,
        id: Uuid,
        device_name: &DeviceName,
    ) -> Result<String, reqwest::Error> {
        self.get_string(&format!("/simulation/{id}/log/{}", device_name.as_str()))
            .await
    }

    pub async fn submit_simulation(
        &self,
        simulation: &simulation::Simulation,
    ) -> Result<SubmittedSimulation, reqwest::Error> {
        self.client
            .put(self.with_path("/simulation"))
            .json(simulation)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
}
