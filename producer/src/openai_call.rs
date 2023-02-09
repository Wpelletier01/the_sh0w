#![allow(dead_code)]


use std::fmt::{Display, Formatter};
use reqwest::Client;
use futures::TryFutureExt;
use json::{object, JsonValue};

use super::logger::{CWARN,CERROR};
use serde::Deserialize;
use serde_json::Value;

/*

curl https://api.openai.com/v1/completions \
-H "Content-Type: application/json" \
-H "Authorization: Bearer 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" \
-d '{"model": "text-davinci-003", "prompt": "Say this is a test", "temperature": 0, "max_tokens": 7}'


*/


//
//
//-------------------------------------------------------------------------------------------------
// Test
//
//
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tcurl_req() {

        //curl_req();

        assert!(true);


    }

    #[test]
    fn invalid_temp_value() {

        use super::super::logger::init;

        init();

        let mut info = PromptRequestInfo {

            model: ModelType::MostAccurate,
            prompt: "Say this is a test".to_string(),
            temperature: Some(3.0),
            top_p: None,
            stop_token: None,
            presence_penalty: None,
            frequency_penalty: None,
            max_word: Some(7),

            nb_response: 1,
            suffix: None,
            logit_bias: None,
        };

        info.body();

    }

}
//
//
// ------------------------------------------------------------------------------------------------
//
pub enum ModelType {

    MostAccurate,
    Accurate,
    FastAndAccurate,
    Fastest,

}
//
impl ModelType {

    fn to_str(&self) -> &str {

        match self {

            Self::MostAccurate => "text-davinci-003",
            Self::Accurate => "text-curie-001",
            Self::FastAndAccurate => "text-babbage-001",
            Self::Fastest => "text-ada-001"

        }


    }


}
//
//
#[derive(Deserialize,Debug)]
struct PromptResponse {

    id:         String,
    object:     String,
    created:    i64,
    model:      String,
    choice:     Value,
    usage:      Value

}
//
//
struct PromptRequestInfo {

    pub prompt:             String,
    pub model:              ModelType,
    pub nb_response:        u16,
    pub max_word:           Option<u16>,
    pub suffix:             Option<String>,
    pub temperature:        Option<f32>,
    pub top_p:              Option<f32>,
    pub stop_token:         Option<Vec<String>>,
    pub presence_penalty:   Option<f32>,
    pub frequency_penalty:  Option<f32>,
    pub logit_bias:         Option<JsonValue>,

}
//
impl PromptRequestInfo {


    fn body(&mut self) -> String {

        let max_tokens = self.max_word.unwrap_or(16);

        let mut body = object!{

            model:      self.model.to_str(),
            prompt:     self.prompt.to_string(),
            max_tokens: max_tokens,
            n:          self.nb_response,


        };


        if let Some(val) = &self.suffix {
            body.insert("suffix",val.to_string()).map_err(|e|
                {
                    CERROR(&format!("unable to add the parameter 'suffix' because of {e}"));

                }

            ).unwrap();

        }


        if let Some(val) = &self.stop_token {

            if val.is_empty() {

                CWARN("You pass an empty vec so nothing will be add to the request");

            } else {
                body.insert("stop",valid_stop_token(val)).map_err(|e|
                    {
                        CERROR(&format!("unable to add the parameter 'stop' because of {e}"));
                    }
                ).unwrap();

            }


        }



        if self.temperature.is_some() && self.top_p.is_some() {

            CWARN("Cannot passed a temperature and top_p parameter");
            CWARN("We will use the temperature parameter value");

            self.top_p = None;

            body.insert("temperature",valid_temp_parameter(self.temperature.unwrap()))
                .map_err(|e| {

                    CERROR(&format!("unable to add the parameter 'temperature' because of {e}"));


                }).unwrap();

        } else if self.temperature.is_none() && self.top_p.is_some() {

            body.insert("top_p",valid_top_p_parameter(self.top_p.unwrap()))
                .map_err(|e| {

                    let mess = format!("unable to add the parameter 'top_p' because of {e}");

                    CERROR(&mess);

                    }
                ).unwrap();

        } else if self.temperature.is_some() && self.top_p.is_none() {

            body.insert("temperature",valid_temp_parameter(self.temperature.unwrap()))
                .map_err(|e| {

                    CERROR(&format!("unable to add the parameter 'temperature' because of {e}"));

                }).unwrap();

        } else {

            CWARN("Should passed at least one of the temperature and top_p parameters");
            CWARN("We will use the temperature parameter value");

            body.insert("temperature",1.0)
                .map_err(|e| {

                    CERROR(&format!("unable to add the parameter 'temperature' because of {e}"));

                    }
                ).unwrap();


        }

        if let Some(val) = self.presence_penalty {

            body.insert("presence_penalty",validate_penalty(val)).map_err(|e|
                {

                    CERROR(&format!("Unable to add parameter 'presence_penalty' because of {e}"));

                }

            ).unwrap();


        }

        if let Some(val) = self.frequency_penalty {

            body.insert("frequency_penalty",validate_penalty(val))
                .map_err(|e|
                CERROR(&format!("Unable to add parameter 'frequency_penalty' because of {e}"))
            ).unwrap();


        }

        if let Some(val) = &self.logit_bias {

            match val {

                JsonValue::Object(obj) => {

                    body.insert("logit_bias",obj.pretty(2))
                        .map_err(|e|
                            CERROR(&format!("Unable to add parameter 'logit_bias' because of {e}"))

                    ).unwrap();

                },

                _ => {

                    CERROR("the parameter 'logit_bias' can only be a json object");

                }


            }

        }


        body.to_string()


    }



}
//
//
// ------------------------------------------------------------------------------------------------
// Validate function
//

fn valid_temp_parameter(temperature:f32) -> f32 {

    if !(0.0..=2.0).contains(&temperature) {

        CERROR("The parameters 'temperature' must be a value between 0 and 2");
        CWARN("The default temperature value of '1' will be pass");

        return 1.0;

    }

    temperature

}
//
//
fn valid_top_p_parameter(top_p:f32) -> f32 {

    if !(0.0..=1.0).contains(&top_p) {

        CERROR("The parameters 'top_p' must be a value between 0 and 1");
        CWARN("The default 'top_p' value of '1' will be pass");

        return 1.0;

    }

    top_p
}
//
fn valid_stop_token(list:&[String]) -> &[String] {

    if list.len() > 4 {

        CERROR("The 'stop' parameter can containt up to 4 token");
        CWARN("the first four token will be used");

        return &list[..4];

    }

    list

}
//
//
fn validate_penalty(penalty:f32) -> f32 {

    if !(-2.0..=2.0).contains(&penalty) {

        CERROR("The 'penalty' parameter value must be between -2 and 2");
        CWARN("The default value of 0 will be send");

        return 1.0;

    }

    penalty

}

//
//
// ------------------------------------------------------------------------------------------------
// Connection
//
struct Connection {

    client: Client

}
//
impl Connection {

    fn init() -> Self { Self { client: Client::new() } }

    async fn send_prompt(&self,body:String) -> Result<(), Box<dyn std::error::Error>> {

        let response = self.client
            .post("https://api.openai.com/v1/completions")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", env!("OPENAI_KEY")))
            .body(body)
            .send()
            .await?
            .json::<PromptResponse>()
            .await?;



   
        Ok(())


    }


}
    



//
//









/*
async fn curl_req() -> Result<(), Box<dyn std::error::Error>>  {
    let body = r#"{
           "model": "text-davinci-003",
           "prompt": "Say this is a test",
           "temperature": 0,
           "max_tokens": 7
       }"#;

    let client = Client::new();

    let res = client
        .post("https://api.openai.com/v1/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", env!("OPENAI_KEY")))
        .body(body)
        .send()
        .await?;


    println!("{:?}",res);

    Ok(())
}
*/






