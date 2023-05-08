use std::io::Cursor;

use amarkl::{AmarkReader, AmarkToken};

#[test]
fn reference_holds() {
    let expected = [
        AmarkToken::ItemName(b"TopLevel"),
        AmarkToken::ContainerStart,
        AmarkToken::ItemName(b"Text"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"Hello World"),
        AmarkToken::EmptyLine,
        AmarkToken::Text(b"HI!"),
        AmarkToken::BlockEnd,
        AmarkToken::ItemName(b"gb"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"A green box"),
        AmarkToken::BlockEnd,
        AmarkToken::ItemName(b"bb"),
        AmarkToken::BlockStart,
        AmarkToken::ItemName(b"l"),
        AmarkToken::ItemEnd,
        AmarkToken::Text(b"A blue box"),
        AmarkToken::EscapeSequence(b'n'),
        AmarkToken::Text(b"Text after break"),
        AmarkToken::BlockEnd,
        AmarkToken::ItemName(b"additional"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"Additional text"),
        AmarkToken::ItemName(b"br"),
        AmarkToken::ParamsStart,
        AmarkToken::Text(b"3"),
        AmarkToken::ParamsEnd,
        AmarkToken::ItemEnd,
        AmarkToken::Text(b"Another additional text"),
        AmarkToken::BlockEnd,
        AmarkToken::ItemName(b"br"),
        AmarkToken::ParamsStart,
        AmarkToken::Text(b"5"),
        AmarkToken::ParamsEnd,
        AmarkToken::ItemEnd,
        AmarkToken::ItemName(b"l"),
        AmarkToken::ItemEnd,
        AmarkToken::ItemName(b"repeat"),
        AmarkToken::ParamsStart,
        AmarkToken::Text(b"5"),
        AmarkToken::ParamsEnd,
        AmarkToken::BlockStart,
        AmarkToken::Text(b"laa"),
        AmarkToken::BlockEnd,
        AmarkToken::ItemName(b"yb"),
        AmarkToken::BlockStart,
        AmarkToken::EscapeSequence(b's'),
        AmarkToken::ParamsStart,
        AmarkToken::Text(b"4"),
        AmarkToken::ParamsEnd,
        AmarkToken::Text(b"A yellow box"),
        AmarkToken::ItemName(b"bb"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"A blue box in a yellow box"),
        AmarkToken::BlockEnd,
        AmarkToken::BlockEnd,
        AmarkToken::ItemName(b"rb"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"A red box"),
        AmarkToken::EscapeSequence(b'n'),
        AmarkToken::ParamsStart,
        AmarkToken::Text(b"4"),
        AmarkToken::ParamsEnd,
        AmarkToken::ItemName(b"bb"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"A blue box in a red box"),
        AmarkToken::BlockEnd,
        AmarkToken::Text(b"Text after"),
        AmarkToken::EmptyLine,
        AmarkToken::ItemName(b"boo"),
        AmarkToken::ParamsStart,
        AmarkToken::Text(b"hello"),
        AmarkToken::ParamsEnd,
        AmarkToken::BlockStart,
        AmarkToken::Text(b"world"),
        AmarkToken::EmptyLine,
        AmarkToken::BlockEnd,
        AmarkToken::Text(b"red box"),
        AmarkToken::BlockEnd,
        AmarkToken::ItemName(b"Text"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"Bye World"),
        AmarkToken::BlockEnd,
        AmarkToken::ContainerEnd,
        AmarkToken::End,
    ];

    let mut aml_reader = AmarkReader::new();
    let mut source = Cursor::new(include_bytes!("../example_files/simple_reference.amark"));
    let mut expected_iter = expected.into_iter();

    while let Some(expected_token) = expected_iter.next() {
        let (got_token, line) = aml_reader.parse_next_get_cur_line(&mut source);
        let got_token =
            got_token.unwrap_or_else(|e| panic!("Failure while parsing on line {}: {:?}", line, e));

        assert_eq!(
            got_token, expected_token,
            "Unexpected token! Expected: {:?}, got: {:?} on line {}",
            expected_token, got_token, line
        );
    }

    assert!(expected_iter.next().is_none());
}

#[test]
fn functions_test() {
    let expected = [
        AmarkToken::ItemName(b"p"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"hi"),
        AmarkToken::BlockEnd,
        AmarkToken::ItemName(b"br"),
        AmarkToken::ParamsStart,
        AmarkToken::Text(b"1"),
        AmarkToken::ParamsEnd,
        AmarkToken::ItemEnd,
        AmarkToken::ItemName(b"p"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"Hello"),
        AmarkToken::BlockEnd,
        AmarkToken::ItemName(b"br"),
        AmarkToken::ParamsStart,
        AmarkToken::Text(b"5"),
        AmarkToken::ParamsEnd,
        AmarkToken::ItemEnd,
        AmarkToken::ItemName(b"p"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"Hello"),
        AmarkToken::ItemName(b"br"),
        AmarkToken::ParamsStart,
        AmarkToken::Text(b"3"),
        AmarkToken::ParamsEnd,
        AmarkToken::ItemEnd,
        AmarkToken::EmptyLine,
        AmarkToken::ItemName(b"ro"),
        AmarkToken::ParamsStart,
        AmarkToken::Text(b"hi,7"),
        AmarkToken::ParamsEnd,
        AmarkToken::BlockStart,
        AmarkToken::Text(b"hello"),
        AmarkToken::EmptyLine,
        AmarkToken::ItemName(b"p"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"hi"),
        AmarkToken::EmptyLine,
        AmarkToken::ItemName(b"ri"),
        AmarkToken::ParamsStart,
        AmarkToken::Text(b"good day"),
        AmarkToken::ParamsEnd,
        AmarkToken::BlockStart,
        AmarkToken::Text(b"bye"),
        AmarkToken::EmptyLine,
        AmarkToken::BlockEnd,
        AmarkToken::BlockEnd,
        AmarkToken::EmptyLine,
        AmarkToken::BlockEnd,
        AmarkToken::EmptyLine,
        AmarkToken::Text(b"World"),
        AmarkToken::BlockEnd,
        AmarkToken::End,
    ];

    let mut aml_reader = AmarkReader::new();
    let mut source = Cursor::new(include_bytes!("../example_files/functions.amark"));
    let mut expected_iter = expected.into_iter();

    while let Some(expected_token) = expected_iter.next() {
        let (got_token, line) = aml_reader.parse_next_get_cur_line(&mut source);
        let got_token =
            got_token.unwrap_or_else(|e| panic!("Failure while parsing on line {}: {:?}", line, e));

        assert_eq!(
            got_token, expected_token,
            "Unexpected token! Expected: {:?}, got: {:?} on line {}",
            expected_token, got_token, line
        );
    }

    assert!(expected_iter.next().is_none());
}

#[test]
fn large_file_works() {
    let expected = [
        AmarkToken::ItemName(b"TopLevel"),
        AmarkToken::ContainerStart,
        AmarkToken::ItemName(b"Text"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"Hello World"),
        AmarkToken::EmptyLine,
        AmarkToken::Text(b"HI!"),
        AmarkToken::BlockEnd,
        AmarkToken::ItemName(b"gb"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"A green box"),
        AmarkToken::BlockEnd,
        AmarkToken::ItemName(b"bb"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"A blue box"),
        AmarkToken::EscapeSequence(b'n'),
        AmarkToken::Text(b"Text after break"),
        AmarkToken::BlockEnd,
        AmarkToken::ItemName(b"additional"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"Additional text"),
        AmarkToken::BlockEnd,
        AmarkToken::ItemName(b"yb"),
        AmarkToken::BlockStart,
        AmarkToken::EscapeSequence(b's'),
        AmarkToken::ParamsStart,
        AmarkToken::Text(b"4"),
        AmarkToken::ParamsEnd,
        AmarkToken::Text(b"A yellow box"),
        AmarkToken::ItemName(b"bb"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"A blue box in a yellow box"),
        AmarkToken::BlockEnd,
        AmarkToken::BlockEnd,
        AmarkToken::ItemName(b"l"),
        AmarkToken::ItemEnd,
        AmarkToken::ItemName(b"br"),
        AmarkToken::ParamsStart,
        AmarkToken::Text(b"4"),
        AmarkToken::ParamsEnd,
        AmarkToken::ItemEnd,
        AmarkToken::ItemName(b"rb"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"A red box"),
        AmarkToken::EscapeSequence(b'n'),
        AmarkToken::ParamsStart,
        AmarkToken::Text(b"4"),
        AmarkToken::ParamsEnd,
        AmarkToken::ItemName(b"bb"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"A blue box in a red box"),
        AmarkToken::BlockEnd,
        AmarkToken::Text(b"Text after"),
        AmarkToken::EmptyLine,
        AmarkToken::Text(b"red box"),
        AmarkToken::BlockEnd,
        AmarkToken::ItemName(b"Text"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"Bye World"),
        AmarkToken::BlockEnd,
        AmarkToken::ItemName(b"ro"),
        AmarkToken::ParamsStart,
        AmarkToken::Text(b"5"),
        AmarkToken::ParamsEnd,
        AmarkToken::BlockStart,
        AmarkToken::Text(b"hello"),
        AmarkToken::EmptyLine,
        AmarkToken::ItemName(b"w"),
        AmarkToken::ItemEnd,
        AmarkToken::ItemName(b"br"),
        AmarkToken::ParamsStart,
        AmarkToken::Text(b"6"),
        AmarkToken::ParamsEnd,
        AmarkToken::ItemEnd,
        AmarkToken::BlockEnd,
        AmarkToken::ItemName(b"LargeText"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Convallis convallis tellus id interdum velit laoreet id. Vel pretium lectus quam id leo in vitae turpis massa. Egestas fringilla phasellus faucibus scelerisque eleifend donec pretium. Hendrerit gravida rutrum quisque non. Eget mi proin sed libero enim sed faucibus. Mattis molestie a iaculis at. Amet aliquam id diam maecenas ultricies. Elementum tempus egestas sed sed risus pretium quam. Elit pellentesque habitant morbi tristique senectus. Eget mi proin sed libero enim. Urna neque viverra justo nec ultrices dui. Cursus sit amet dictum sit amet justo donec enim diam. Ultricies lacus sed turpis tincidunt. Mi eget mauris pharetra et ultrices neque ornare aenean euismod."),
        AmarkToken::EmptyLine,
        AmarkToken::Text(b"Dolor sit amet consectetur adipiscing elit ut. Dui ut ornare lectus sit amet est. Nec tincidunt praesent semper feugiat nibh. Et tortor at risus viverra adipiscing at in tellus. Ante metus dictum at tempor commodo. Posuere morbi leo urna molestie at elementum eu. Turpis in eu mi bibendum neque egestas congue quisque. Placerat duis ultricies lacus sed turpis tincidunt id aliquet. Augue lacus viverra vitae congue eu. Nullam vehicula ipsum a arcu cursus vitae congue mauris. Vivamus arcu felis bibendum ut tristique et egestas quis. Viverra aliquet eget sit amet tellus cras adipiscing enim eu. Dolor morbi non arcu risus quis varius quam. A diam maecenas sed enim ut. Nulla facilisi etiam dignissim diam quis. Aenean vel elit scelerisque mauris pellentesque pulvinar pellentesque habitant morbi. Ac tortor dignissim convallis aenean et tortor at. Nullam eget felis eget nunc lobortis mattis. Adipiscing elit pellentesque habitant morbi tristique senectus et netus. Elementum tempus egestas sed sed risus pretium quam."),
        AmarkToken::EmptyLine,
        AmarkToken::Text(b"Odio ut enim blandit volutpat maecenas volutpat blandit aliquam etiam. Pellentesque eu tincidunt tortor aliquam nulla. Euismod in pellentesque massa placerat. Morbi leo urna molestie at elementum eu facilisis sed odio. Et egestas quis ipsum suspendisse ultrices gravida dictum fusce ut. Tellus at urna condimentum mattis. Auctor eu augue ut lectus. Ullamcorper dignissim cras tincidunt lobortis feugiat. Tellus elementum sagittis vitae et leo. Ornare arcu dui vivamus arcu felis bibendum."),
        AmarkToken::EmptyLine,
        AmarkToken::Text(b"Ullamcorper morbi tincidunt ornare massa. Magnis dis parturient montes nascetur ridiculus mus mauris vitae ultricies. Egestas quis ipsum suspendisse ultrices gravida dictum. Arcu cursus vitae congue mauris rhoncus aenean vel elit scelerisque. Ornare arcu odio ut sem nulla pharetra diam sit amet. A lacus vestibulum sed arcu non. Quis enim lobortis scelerisque fermentum dui faucibus in. Ultrices in iaculis nunc sed augue lacus viverra vitae. Sit amet mattis vulputate enim nulla aliquet porttitor lacus. Aliquam sem fringilla ut morbi tincidunt augue interdum velit. Adipiscing elit pellentesque habitant morbi tristique senectus et. Molestie at elementum eu facilisis sed. Arcu cursus vitae congue mauris rhoncus aenean vel. Egestas purus viverra accumsan in nisl nisi scelerisque. Nibh mauris cursus mattis molestie a iaculis. Viverra ipsum nunc aliquet bibendum. Erat pellentesque adipiscing commodo elit. Dictum at tempor commodo ullamcorper a lacus vestibulum. Pharetra vel turpis nunc eget lorem dolor sed viverra ipsum. Id eu nisl nunc mi ipsum faucibus vitae."),
        AmarkToken::EmptyLine,
        AmarkToken::Text(b"Et odio pellentesque diam volutpat commodo sed egestas. Id velit ut tortor pretium viverra suspendisse. Sem integer vitae justo eget magna fermentum iaculis. Dolor magna eget est lorem ipsum dolor sit. Porta nibh venenatis cras sed felis eget velit aliquet sagittis. Et ligula ullamcorper malesuada proin libero nunc consequat interdum varius. Mauris sit amet massa vitae tortor condimentum lacinia. Adipiscing bibendum est ultricies integer quis auctor elit sed vulputate. Pulvinar neque laoreet suspendisse interdum consectetur libero. Scelerisque fermentum dui faucibus in ornare quam viverra orci. Bibendum at varius vel pharetra vel turpis nunc. Montes nascetur ridiculus mus mauris vitae ultricies leo integer malesuada. Maecenas ultricies mi eget mauris. Eget arcu dictum varius duis at consectetur. Ac tortor vitae purus faucibus ornare. Purus viverra accumsan in nisl nisi scelerisque. Mauris a diam maecenas sed enim ut sem viverra aliquet. Etiam erat velit scelerisque in dictum non. In vitae turpis massa sed. Donec enim diam vulputate ut.  "),
        AmarkToken::BlockEnd,
        AmarkToken::ItemName(b"DeepNesting"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"T0"),
        AmarkToken::ItemName(b"d0"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"T1"),
        AmarkToken::ItemName(b"d1"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"T2"),
        AmarkToken::ItemName(b"d2"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"T3"),
        AmarkToken::ItemName(b"d3"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"T4"),
        AmarkToken::ItemName(b"d4"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"ahahahaha"),
        AmarkToken::ItemName(b"ah"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"Ahahahahah"),
        AmarkToken::BlockEnd,
        AmarkToken::Text(b"T5"),
        AmarkToken::ItemName(b"d5"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"T6"),
        AmarkToken::ItemName(b"d6"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"T7"),
        AmarkToken::ItemName(b"d7"),
        AmarkToken::BlockStart,
        AmarkToken::ItemName(b"DD"),
        AmarkToken::BlockStart,
        AmarkToken::ItemName(b"DDD"),
        AmarkToken::BlockStart,
        AmarkToken::ItemName(b"DDDD"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"l;"),
        AmarkToken::Text(b"br(4);"),
        AmarkToken::ItemName(b"l"),
        AmarkToken::ItemEnd,
        AmarkToken::ItemName(b"br"),
        AmarkToken::ParamsStart,
        AmarkToken::Text(b"4"),
        AmarkToken::ParamsEnd,
        AmarkToken::ItemEnd,
        AmarkToken::ItemName(b"DDDDD"),
        AmarkToken::BlockStart,
        AmarkToken::ItemName(b"DDDDDD"),
        AmarkToken::BlockStart,
        AmarkToken::Text(b"aahh thats deep"),
        AmarkToken::BlockEnd,
        AmarkToken::BlockEnd,
        AmarkToken::BlockEnd,
        AmarkToken::BlockEnd,
        AmarkToken::BlockEnd,
        AmarkToken::EmptyLine,
        AmarkToken::BlockEnd,
        AmarkToken::BlockEnd,
        AmarkToken::BlockEnd,
        AmarkToken::BlockEnd,
        AmarkToken::BlockEnd,
        AmarkToken::BlockEnd,
        AmarkToken::BlockEnd,
        AmarkToken::BlockEnd,
        AmarkToken::BlockEnd,
        AmarkToken::ContainerEnd,
        AmarkToken::End,
    ];

    let mut aml_reader = AmarkReader::new();
    let mut source = Cursor::new(include_bytes!("../example_files/large_file.amark"));
    let mut expected_iter = expected.iter();
    let mut block_starts = 0u32;
    let mut block_ends = 0u32;

    while let Some(expected_token) = expected_iter.next() {
        let (got_token, line) = aml_reader.parse_next_get_cur_line(&mut source);
        let got_token =
            got_token.unwrap_or_else(|e| panic!("Failure while parsing on line {}: {:?}", line, e));
        if let AmarkToken::BlockStart = got_token {
            block_starts += 1;
        }

        if let AmarkToken::BlockEnd = got_token {
            block_ends += 1;
        }

        assert_eq!(
            &got_token, expected_token,
            "Unexpected token! Expected: {:?}, got: {:?} on line {}",
            expected_token, got_token, line
        );
    }

    assert!(expected_iter.next().is_none());
    assert_eq!(block_starts, block_ends);
}
