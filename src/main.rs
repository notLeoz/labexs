use std::{process::ExitCode, env, io::{ Write}, fs::{File, create_dir}};
use reqwest::{header, Client};
use scraper::{Html, Selector};

const LINK : &str = "https://elearning.studenti.math.unipd.it/labs/course/view.php?id=25";

fn create_client_with_default_headers(moodle_session: &str) -> Client {
    let mut headers = header::HeaderMap::new();
    headers.insert(header::COOKIE, header::HeaderValue::from_str(&format!("MoodleSession={moodle_session}")).expect("Should be valid"));
    let client = reqwest::ClientBuilder::new().default_headers(headers).build().map_err(|err| {
        eprintln!("|ERROR|  Could not create ClientBuilder: {err}")
    }).unwrap();
    client
}

async fn verify_moodle_session(moodle_session: &str) -> Result<(),()> {
    let client = create_client_with_default_headers(moodle_session);
    match client.get(LINK).send().await{
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

async fn get_lab_links(lab: &str,moodle_session: &str) -> Result<Vec<String>, ()> {
    let full_lab = format!("Laboratorio {lab}") ;
    let client = create_client_with_default_headers(moodle_session);
    println!("|INFO| Created ClientBuilder");
    let res = client.get(LINK).send().await.map_err(|err| {
        eprintln!("|ERROR| Could not make the get request: {err}");
        eprintln!("|INFO|  MoodleSession is probably expired!");
    })?.text().await.map_err(|err| {
        eprintln!("|ERROR| Could not read the response: {err}");
    })?;
    println!("{}", format!("|INFO| Made the get request to {LINK}"));
    let doc = Html::parse_document(&res);
    let li_lab_selector = Selector::parse(&format!("li[aria-label=\"{full_lab}\"]")).map_err(|err| {
        eprintln!("|ERROR| Could not create li_lab_selector: {err}")
    })?;
    let div_activity_selector = Selector::parse(".activityinstance").map_err(|err| {
        eprintln!("|ERROR| Could not create div_activity_selector: {err}")
    })?;
    let a_selector = Selector::parse("a[onclick=\"\"]").map_err(|err| {
        eprintln!("|ERROR| Could not create the 'anchor' tag selector: {err}")
    })?;
    let li_lab = doc.select(&li_lab_selector).next().ok_or_else(|| {
        eprintln!("|ERROR| Could not select li tags from documents")
    })?;
    let div_lab = li_lab.select(&div_activity_selector);
    let mut links_to_exs: Vec<String> = Vec::new();
    println!("|INFO| Parsing Elements...");
    for el1 in div_lab {
        for el2 in el1.select(&a_selector) {
            let src_image = el2.children().next().unwrap().value().as_element().unwrap().attr("src").unwrap().replace("_s/", ""); // si fa il replace finale perchè prendendo il values "as_element()" aggiunge il /_s/
            if src_image.eq(&String::from("https://elearning.studenti.math.unipd.it/labs/theme/image.php/boost/vpl/1591223608/icon")) {
                let link = el2.value().attr("href").unwrap();
                links_to_exs.push(link.to_owned());
            }
        }
    }
    Ok(links_to_exs)
} 

async fn get_lab_exs(links: Vec<String>,moodle_session: &str,lab: &str) -> Result<(),()> {
    println!("|INFO| Starting to read exercises...");
    let client = create_client_with_default_headers(moodle_session);
    //ONLY FIRST LINK FOR TESTING
    let mut counter = 1;
    create_dir(format!("../../Laboratorio {lab}")).map_err(|err| {
        eprintln!("{}",format!("|ERROR| Could not create directory 'Laboratorio {lab}': {err}"));
    })?;
    println!("|INFO| Created directory 'Laboratorio {lab}'");
    for link in links {
        println!("|INFO| Getting infos of {counter}° exercise");
        let res = client.get(link).send().await.map_err(|err| {
            eprintln!("|ERROR| Could not make the get request: {err}");
        })?.text().await.map_err(|err| {
            eprintln!("|ERROR| Could not read the response: {err}");
        })?;
        let doc = Html::parse_document(&res);
        let file_name_selector = Selector::parse("h4[id=\"fileid1\"]").unwrap();
        let file_name = doc.select(&file_name_selector).next().unwrap().inner_html();
        let ex_text = doc.select(&Selector::parse("pre[id=\"codefileid1\"]").unwrap()).next().unwrap().children().next().unwrap().value().as_text().unwrap();
        let mut file = File::create(format!("../../Laboratorio {lab}/").to_owned()+&file_name.to_owned()).map_err(|err| {
            eprintln!("|ERROR| Could not create file named {file_name}: {err}");
        })?;
        println!("|INFO| Created file");
        println!("|INFO| Writing...");
        for el in ex_text.split("\n") {
            file.write_all((el.to_owned()+"\n").as_bytes()).unwrap();
        }
        println!("|INFO| Done.");
        counter+=1;
    }
    println!("");
    println!("|INFO| Successfully created all exercises files!");
    Ok(())
}
fn usage() {
    eprintln!("Usage: ./labexs [LAB_NUMBER] [MOODLE_SESSION]");
    eprintln!("Example:");
    eprintln!("        ./labexs 4 abcdefghijklm      Downloads all the exercises of lab number 4");
    eprintln!("STILL NOT IMPLEMENTED - #In case of expired moodle session it can create a new one if email and password are in the env file#");
}
async fn create_new_moodle_session() -> Result<String,()> {
    println!("|INFO| Retriving email and password from .env");
    //TODO: get the values from env file and create new moodle_session
    Ok("Hello".to_owned())
}
async fn entry() -> Result<(), ()> {
    let mut args = env::args();
    args.next();
    let lab = args.next().ok_or_else(|| {
        eprintln!("|ERROR| Didn't provide the number of lab");
        usage();
    })?;
    let mut moodle_session = args.next().ok_or_else(|| {
        eprintln!("|ERROR| Didn't provide the MoodleSession");
        usage();
    })?;
    println!("|INFO| Checking Moodle Session");
    match verify_moodle_session(&moodle_session).await{
        Ok(_) => {
            eprintln!("|INFO| Moodle Session is valid! Proceeding...")
        },
        Err(_) => {
            eprintln!("|INFO| Moodle session is expired... creating new one"); //TODO: create a new moodle session 
            match create_new_moodle_session().await {
                Ok(data) => moodle_session = data,
                Err(_) => return Err(()),
            }
        },
    };

    let result = match get_lab_links(&lab,&moodle_session).await {
        Ok(data) => get_lab_exs(data,&moodle_session,&lab).await,
        Err(_) => Err(()),
    };
    
    
    result
}
#[tokio::main]
async fn main() -> ExitCode{
    match entry().await{
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => ExitCode::FAILURE,
    }
        
}


