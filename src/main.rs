use std::path::Path;
use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() {
    let mut resp = reqwest::get("http://oasis.caiso.com/oasisapi/SingleZip?queryname=PRC_INTVL_LMP&startdatetime=20250102T07:00-0000&enddatetime=20250102T08:00-0000&version=3&market_run_id=RTM&node=LOGANCIT_LNODED1").await.unwrap();

    let path = Path::new("./data.zip");
    let path1 = Path::new("./data/");
    let mut file = File::create(&path).unwrap();
    let data = resp.bytes().await.unwrap();
    file.write_all(&data).unwrap();

    let zipfile = File::open(path).unwrap();
    let mut zip = zip::ZipArchive::new(zipfile).unwrap();
    zip.extract(path1).unwrap();
    
}
