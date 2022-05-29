use image::GenericImageView;


fn main() {
     // Use the open function to load an image from a Path.
    // `open` returns a `DynamicImage` on success.
    let img = image::open("./images/pikachu.png").unwrap();

    // The dimensions method returns the images width and height.
    println!("dimensions {:?}", img.dimensions());
}
