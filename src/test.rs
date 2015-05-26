use std::io::prelude::*;
use std::fs::{File, OpenOptions};
use std::env::temp_dir;

use super::{FileMap, FileMapMut};

#[test]
fn test_filemap() {
    const LEN: usize = 1024;
    let data = [0xab; LEN];
    let mut filename = temp_dir();
    filename.push("rust-filemap-test");

    {
        let mut file = File::create(&filename).unwrap();
        file.write(&data).unwrap();
    }

    {
        let file = OpenOptions::new()
                .read(true)
                .write(true)
                .truncate(false)
                .open(&filename)
                .unwrap();

        let mut map_w = FileMapMut::new(&file, 0, LEN, true).unwrap();
        let slice = &mut map_w[..];
        assert_eq!(slice, &data[..]);

        slice[0] = 0xcd;
        slice[LEN - 1] = 0xef;
    }

    {
        let file = File::open(&filename).unwrap();
        let map = FileMap::new(&file, 0, LEN).unwrap();
        let slice = &map[..];
        assert_eq!(slice[0], 0xcd);
        assert_eq!(slice[LEN - 1], 0xef);
        assert_eq!(&slice[1..(LEN - 1)], &data[1..(LEN - 1)]);
    }

    // offset % page_size != 0
    {
        let file = File::open(&filename).unwrap();
        let offset = 10;
        let map = FileMap::new(&file, offset, LEN - offset).unwrap();
        assert_eq!(map.len(), LEN - offset);

        let slice = &map[..];
        assert_eq!(slice.len(), LEN - offset);
        assert_eq!(slice[LEN - 1 - offset], 0xef);
        assert_eq!(&slice[1..(LEN - 1 - offset)], &data[(1 + offset)..(LEN - 1)]);
    }
}
