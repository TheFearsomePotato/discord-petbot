use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use image::gif::GifDecoder;
use image::imageops;
use image::io::Reader as ImageReader;
use image::AnimationDecoder;
use image::Frame;

use std::fs::File;

const PETGIF_FILENAME: &'static str = "./assets/pet.gif";
const INVALID_MSG_REPLY: &'static str = "No mentioned user. You must mention a user.";

struct PetGif;

impl TypeMapKey for PetGif {
    type Value = Vec<Frame>;
}

pub async fn prepare_pet(client: &mut Client) {
    // Load and Decode gif
    let file_in = File::open(PETGIF_FILENAME).unwrap();
    let decoder = GifDecoder::new(file_in).unwrap();
    let frames = decoder.into_frames();
    let frames = frames.collect_frames().expect("error decoding pet-gif");

    let mut data = client.data.write().await;
    data.insert::<PetGif>(frames);
}

#[command]
pub async fn pet(ctx: &Context, msg: &Message) -> CommandResult {
    // Check if there are mentions, if not bring error
    if msg.mentions.len() < 1 as usize {
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.content(INVALID_MSG_REPLY);
                m.reference_message(msg);
                m
            })
            .await?;
        return Ok(());
    }

    let mentions = &msg.mentions;

    for user in mentions.iter() {
        let avatar_url = match user.avatar.as_ref() {
            Some(h) => format!(
                "https://cdn.discordapp.com/avatars/{}/{}.png?size=128",
                &user.id, h
            ),
            None => format!(
                "https://cdn.discordapp.com/embed/avatars/{}.png",
                &user.discriminator % 5u16,
            ),
        };
        let username = &user.name;

        // Get the reference to the original gif
        let data = ctx.data.read().await;
        let petgif = data.get::<PetGif>().expect("Expected petgif in TypeMap.");

        // download the file
        let avatar_picture_response: reqwest::Response = reqwest::get(&avatar_url).await?;
        let avatar_picture_data = avatar_picture_response.bytes().await?;

        // create the gif based on the avatar
        let filename = format!("PetThe{}.gif", username);
        let data = create_petgif(&avatar_picture_data[..], petgif).await?;

        msg.channel_id
            .send_message(&ctx.http, |m| {
                //m.content(&avatar_url);
                m.add_file((data.as_slice(), filename.as_str()));
                m.reference_message(msg);
                m
            })
            .await?;
    }

    Ok(())
}

async fn create_petgif(
    avatar_pic: &[u8],
    pet_frames: &Vec<Frame>,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    // Read the avatar pic
    let mut reader = ImageReader::new(std::io::Cursor::new(avatar_pic));
    reader.set_format(image::ImageFormat::Png);

    // Resize the avatar pic
    let mut avatar = imageops::resize(
        &reader.decode()?.to_rgba8(),
        90,
        90,
        image::imageops::FilterType::Triangle,
    );

    // Cut a circle around the avatar
    for (x, y, p) in avatar.enumerate_pixels_mut() {
        if 45 * 45 < ((x as i32) - 45).pow(2) + ((y as i32) - 45).pow(2) {
            p[3] = 0;
        }
    }

    // data of the final gif
    let mut data: Vec<u8> = Vec::new();

    // Define the encoder
    let mut encoder = gif::Encoder::new(&mut data, 112, 112, &[]).unwrap();
    encoder.set_repeat(gif::Repeat::Infinite).unwrap();

    // Go over all of the frames of the original picture
    for (index, hand_frame) in pet_frames.iter().enumerate() {
        // Calculate on offset based on the current frame
        let height_offset: u32 = (4 - (index as i32 - 4).abs()) as u32;

        // The buffer if the new frame
        let mut new_frame: image::RgbaImage = image::RgbaImage::new(112, 112);
        // Write the avatar image first, with the offset
        image::imageops::overlay(&mut new_frame, &avatar, 12, 22 + height_offset * 3);
        // Layer the original gif over that
        image::imageops::overlay(&mut new_frame, hand_frame.buffer(), 0, 0);

        // Convert the buffer to the new frame, speed is quality / speed of the encoder
        let mut new_frame = gif::Frame::from_rgba_speed(112, 112, &mut new_frame, 2);
        // Make the gif not be fucked up
        new_frame.dispose = gif::DisposalMethod::Background;
        new_frame.delay = 3;

        // write the frame to the data vector with the encoder
        encoder.write_frame(&new_frame).unwrap();
    }
    // drop the encoder, so that the data can be returned
    drop(encoder);
    return Ok(data);
}
