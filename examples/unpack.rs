use std::fs::File;
use std::io::{Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1)
        .ok_or("Provide path to a TNEF file (winmail.dat)".to_string())?;
    let mut f = File::open(path)?;
    let mut buf = vec![];
    f.read_to_end(&mut buf)?;

    for a in tnef::read_attachements(&buf)? {
        println!("\
                Title: {:?}\nCreate date: {:?}\nModify date: {:?}\n\
                Data len: {:?}\nMeta len: {:?}\n\
                Transport filename: {:?}\nRendering data: {:?}\n\
                Props len: {:?}\n\
            ",
            a.title,
            a.create_date,
            a.modify_date,
            a.data.len(),
            a.meta.map(|v| v.len()),
            a.transport_filename,
            a.rend_data,
            a.props.len(),
        );
        File::create(a.title)?.write_all(a.data)?;
    }
    Ok(())
}
