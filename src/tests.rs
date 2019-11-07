use super::to_string;
use crypto::digest::Digest;
use crypto::sha2;
use std::fs;

#[test]
fn test_expected_digest() {
    let dir = fs::read_dir("testdata").unwrap();
    for f in dir {
        let f = f.unwrap();
        let metadata = fs::metadata(f.path()).unwrap();
        if metadata.is_dir() {
            continue;
        }
        let input = fs::File::open(f.path()).expect("cannot open input file");
        let res: serde_json::Value =
            serde_json::from_reader(input).expect("cannot deserialize input file");

        // the SHA256 of the JSON files is computed with a new line at the end of the file
        let cj = format!(
            "{}\n",
            to_string(&res).expect("cannot convert to canonical JSON")
        );
        let b = cj.as_bytes();
        let mut hasher = sha2::Sha256::new();
        hasher.input(&b);
        let actual_digest = hasher.result_str();
        let p = f.path();
        let expected_digest = p.file_stem().unwrap();
        assert_eq!(
            actual_digest,
            expected_digest
                .to_str()
                .expect("cannot convert OsStr to str")
        );
    }
}

#[test]
fn test_expected_error() {
    let dir = fs::read_dir("testdata/errors").unwrap();
    for f in dir {
        let f = f.unwrap();
        let metadata = fs::metadata(f.path()).unwrap();
        if metadata.is_dir() {
            continue;
        }
        let input = fs::File::open(f.path()).expect("cannot open input file");
        let res: serde_json::Value =
            serde_json::from_reader(input).expect("cannot deserialize input file");

        match to_string(&res) {
            Ok(_) => assert!(false, "expected error for file {:?}", f.path()),
            Err(_) => assert!(true),
        }
    }
}
