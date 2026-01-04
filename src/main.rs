const DECK_SIZE: usize = 612;
const BORDER_WIDTH: u32 = 36;
const BORDER_COLOR: image::Rgba<u8> = image::Rgba([0, 0, 0, 255]);

struct CardInfo {
    name: String,
    set_code: String,
    collector_number: usize,
    count: usize,
}

impl std::fmt::Debug for CardInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}x \"{}\" {{{}}}[{}]",
            self.count, self.name, self.set_code, self.collector_number
        )
    }
}

#[tokio::main]
async fn main() {
    // parse file
    let infos: Vec<CardInfo> = std::fs::read_to_string("cards.txt")
        .unwrap()
        .lines()
        .filter_map(|line| {
            if line.is_empty() {
                None
            } else {
                let (count, line) = line.split_once(' ').unwrap();
                let (line, collector_number) = line.rsplit_once(' ').unwrap();
                let (name, line) = line.rsplit_once(' ').unwrap();
                let set_code = line.strip_prefix('(').unwrap().strip_suffix(')').unwrap();
                Some(CardInfo {
                    name: String::from(name),
                    set_code: String::from(set_code),
                    collector_number: collector_number.parse::<usize>().unwrap(),
                    count: count.parse::<usize>().unwrap(),
                })
            }
        })
        .collect();

    println!("parsed {} unique cards", infos.len());
    println!(
        "parsed {} total cards",
        infos.iter().map(|info| info.count).sum::<usize>()
    );

    println!("deleting output directory");
    let _ = std::fs::remove_dir_all("output");

    println!("processing images");
    let mut index = 0;
    for info in infos {
        // fetch card
        let card = scryfall::card::Card::set_and_number(&info.set_code, info.collector_number)
            .await
            .unwrap();
        assert_eq!(card.name, info.name);

        // fetch image
        let image = card.image_uris.unwrap().png.unwrap();
        let image = reqwest::get(image).await.unwrap().bytes().await.unwrap();
        let image = std::io::Cursor::new(image);
        let image = image::ImageReader::with_format(image, image::ImageFormat::Png);
        let image = image.decode().unwrap();
        assert_eq!((image.width(), image.height()), (745, 1040));

        // process image
        let original = image;
        let mut image = image::ImageBuffer::from_pixel(
            original.width() + BORDER_WIDTH * 2,
            original.height() + BORDER_WIDTH * 2,
            BORDER_COLOR,
        );
        for x in 0..original.width() {
            for y in 0..original.height() {
                use image::{GenericImageView, Pixel};
                let mut color = BORDER_COLOR;
                color.blend(&original.get_pixel(x, y));
                image.put_pixel(BORDER_WIDTH + x, BORDER_WIDTH + y, color);
            }
        }

        // save images
        for _ in 0..info.count {
            let dir = format!("output/deck{}", index / DECK_SIZE);
            let file = format!("{dir}/image{}.png", index % DECK_SIZE);

            std::fs::create_dir_all(dir).unwrap();
            image.save(file).unwrap();

            index += 1;
        }
    }

    let card_back = image::ImageBuffer::from_pixel(
        745 + BORDER_WIDTH * 2,
        1040 + BORDER_WIDTH * 2,
        image::Rgb::<u8>([255, 255, 255]),
    );
    card_back
        .save_with_format("output/back.png", image::ImageFormat::Png)
        .unwrap();
    index += 1;

    println!("saved {} images", index);
}
