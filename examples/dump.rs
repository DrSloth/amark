use std::{
    env,
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
};

use amarkl::{AmarkReader, AmarkToken};

fn main() -> io::Result<()> {
    if let Some(path) = env::args_os().nth(1) {
        let mut reader = BufReader::new(File::open(path)?);
        print_all(&mut reader);
    } else {
        print_all(&mut io::stdin().lock());
    }

    Ok(())
}

fn print_all(reader: &mut impl BufRead) {
    let mut aml_reader = AmarkReader::new();
    let mut stdout = BufWriter::with_capacity(4_000_000, io::stdout().lock());

    loop {
        let tok = match aml_reader.parse_next_get_cur_line(reader) {
            (Ok(tok), _) => tok,
            (Err(e), line) => {
                // Flush before showing the error
                let _ = stdout.flush();
                panic!("Failure parsing on line {}: {}", line, e)
            }
        };

        let _ = tok.dump(&mut stdout);
        let _ = stdout.write_all(b"\n");

        if let AmarkToken::End = tok {
            break;
        }
    }
}
