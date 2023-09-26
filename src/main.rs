use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::time::{Duration, Instant};
use serde::de::Visitor;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Item {
    name: String,
    source: String,
}


struct Speedtest{
}
impl Speedtest {
    fn getServerLists() -> [&'static str; 4] {
        [
            "://www.speedtest.net/speedtest-servers-static.php",
            "http://c.speedtest.net/speedtest-servers-static.php",
            "://www.speedtest.net/speedtest-servers.php",
            "http://c.speedtest.net/speedtest-servers.php",
        ]
    }
    async fn getServerEndpoints() -> Result<Vec<Server>, reqwest::Error> {
        let mut Endpoints = Vec::new();


        println!("--------------------------------------------------------");
        for config in Speedtest::getServerLists() {
            let resp = reqwest::get(reqUtils::build_request(config.to_string(),None))
                .await?;

            let str  = resp.text().await.unwrap();

            let setting:Settings = quick_xml::de::from_str(&*str).unwrap();

            for server in setting.server_config.servers {
               println!("{}",server.uri.to_string());
               Endpoints.push(server);
            }
            println!("--------------------------------------------------------");
        }

        Ok(Endpoints)
    }
}

struct reqUtils{
}

impl reqUtils {
    fn build_request(mut url:String,secure:Option<bool>) -> String{
        if (url.starts_with(":")){
            let scheme = {
                match secure {
                    Some(value) if value == true => {"https"}
                    Some(value) if value == false => {"http"}
                    _ => {
                        //some default value
                        "https"
                    }
                }
            };

            url.insert_str(0,scheme);
        }
        if (url.starts_with(":")) {
            //possible settings for secure and non secure
            url.insert_str(0,"https");
        }

        url
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Settings {
    #[serde(rename = "servers")]
    server_config: Servers
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "servers")]
struct Servers {
    #[serde(rename = "server")]
    servers: Vec<Server>
}

//<server url="http://speedtest.gbg.telenor.se:8080/speedtest/upload.php" lat="57.7089" lon="11.9746" name="Göteborg" country="Sweden" cc="SE" sponsor="Telenor AB" id="35925" host="speedtest.gbg.telenor.se:8080"/>
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Server{
    #[serde(rename = "@id")]
    id:u32,
    //#[serde(rename = "@host")]
    //host: String,
    //#[serde(rename = "@url")]
    //url:String,
    //#[serde(rename = "@lat")]
    //lat: f32,
    //#[serde(rename = "@lon")]
    //lon:f32,
    #[serde(flatten)]
    coordinate:Coordinate,
    #[serde(rename = "@name")]
    name:String,
    #[serde(rename = "@country")]
    country:String,
    #[serde(rename = "@cc")]
    cc: String,
    #[serde(rename = "@sponsor")]
    sponsor: String,
    #[serde(skip)]
    ping: Duration,
    #[serde(with = "UriParser", rename = "@url")]
    uri:Uri
}

mod UriParser{
    use hyper::http::uri::InvalidUri;
    use hyper::Uri;
    use serde::{Deserialize, Deserializer, Serializer};
    use serde::de::Error;

    pub fn serialize<S>(
        uri: &Uri,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_str(&uri.to_string())
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Uri, D::Error>
        where
            D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?.parse::<Uri>().map_err( Error::custom)
    }
}

use hyper::http::Uri;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "coordinate")]
struct Coordinate {
    #[serde(rename = "@lat")]
    lat: f64,
    #[serde(rename = "@lon")]
    lon: f64,
}

impl Server {
    async fn testPing(&mut self) -> Result<Duration, reqwest::Error> {
        let mut test_url = format!("{}://{}/latency.txt",
                                   self.uri.scheme_str().expect("No scheme detected"),
                                   self.uri.authority().expect("No authority detected"));

        //test_url = reqUtils::build_request(test_url,None);

        let start = Instant::now();

        let resp = reqwest::get(test_url)
            .await?;

        self.ping = Instant::now().duration_since(start);

        Ok(self.ping)
    }
}


#[tokio::main]
async fn main()-> Result<(), Box<dyn std::error::Error + Send + Sync>>  {
    let vec = Speedtest::getServerEndpoints().await.expect("TODO: panic message");

    for mut server in vec {
        let duration = server.testPing().await.unwrap_or(Duration::new(0, 0));
        println!("{} has {} ms og ping",server.sponsor,duration.as_millis());
    }

    Ok(())
}