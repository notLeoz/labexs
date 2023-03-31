use std::{process::ExitCode, env};

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
    let res = match client.get(LINK).send().await{
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    };
    res
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
fn get_lab_exs(links: Vec<String>) -> Result<(),()> {
    Ok(())
}
fn usage() {
    eprintln!("Usage: ./labexs [LAB_NUMBER] [MOODLE_SESSION]");
    eprintln!("Example:");
    eprintln!("        ./labexs 4 abcdefghijklm      Downloads all the exercises of lab number 4");
    eprintln!("IN CASE OF EXPIRED MOODLE SESSION IT CAN CREATE A NEW ONE IF EMAIL AND PASSWORDS ARE IN THE ENV FILE")
}

async fn entry() -> Result<(), ()> {
    let mut args = env::args();
    args.next();
    let lab = args.next().ok_or_else(|| {
        eprintln!("|ERROR| Didn't provide the number of lab");
        usage();
    })?;
    let moodle_session = args.next().ok_or_else(|| {
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
            return Err(());
        },
    };


    let result = match get_lab_links(&lab,&moodle_session).await {
        Ok(data) => get_lab_exs(data),
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


