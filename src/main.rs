
use reqwest::Client;
use dotenv::dotenv;
use serde::{Deserialize, Serialize}; 
//provides macros for serializing and deserializing data, converting Rust structs to JSON
//and vise versa
use std::io::stdin;  //used to read userinput from the terminal
use std::env; //used to access envt variables

//adding structs for OpenAI communication
//we define some structures that will help us manage the data we send to and receive 
//from the OpenAI API.these structs will represent the request and response formats
//expected by the API



#[derive(Serialize)]
struct OpenAIChatRequest {
    model: String,  //specifies the OPenAI model to use
    messages: Vec<Message>,
    //list of Message structs representing the coversation history
}

#[derive(Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIChatResponse {
    choices: Vec<ChatChoice>, //list og ChatChoice structs containing the AIs response
}
//the one above represents the response from the OpenAI API.



#[derive(Serialize, Deserialize)]
struct Conversation {
    messages: Vec<Message>, //alist of Message structs
}
//represents the entire conversation history




#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatMessage {
    content: String,
}


#[tokio::main]
async fn main() -> Result<(), String> {
    dotenv().ok();  //pulling envt variables from .env

    let client = Client::new();
    let mut conversation = Conversation {
        messages: Vec::new(),
    };

    println!("/nWhat do you want to talk about today?\n");
    loop {
        println!("You:");
        let mut question = String::new();
        stdin().read_line(&mut question).map_err(|e| e.to_string())?;
        //above converts errors from one type to another.
        let question = question.trim();


        if question.eq_ignore_ascii_case("exit"){
            break;
        }

        conversation.messages.push(Message {
            role: "user".to_string(),
            content:question.to_string(),
        });

        let response = ask_openai(&client, &mut conversation).await?;
        println!("\nAI: {}", response);
    }

    Ok(()) //returning case here
}


//crafting the ask_openai func
//sends the current state of the coversation to OpenAI and retrieves the AI response

async fn ask_openai(client: &Client,conversation: &mut Conversation) -> Result<String, String> {
    let request_body = OpenAIChatRequest {
        model: "gpt-3.5-turbo".to_string(), // Specify the OpenAI model you want here
        messages: conversation.messages.clone(),
    };

    let response = client 
        .post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header(
            "Authorization",
            format!("Bearer {}", env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set")),
        )
        .json(&request_body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let response_body = response
        .json::<OpenAIChatResponse>()
        .await
        .map_err(|e| e.to_string())?;
    if let  Some(choice) = response_body.choices.last() {
        conversation.messages.push(Message {
            role: "assistant".to_string(),
            content: choice.message.content.clone(),
        });
        Ok(choice.message.content.clone()) //we need to clone the String value to avoid moving it
        //out of the reference.
        //this creates a new copy of the String and returns it, avoiding the move error
    } else {
        Err("No response from AI".to_string())
    }

}