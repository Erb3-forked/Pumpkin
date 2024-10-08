use pumpkin_config::BASIC_CONFIG;
use pumpkin_core::math::{
    get_section_cord, position::WorldPosition, vector2::Vector2, vector3::Vector3,
};
use pumpkin_protocol::client::play::{CCenterChunk, CUnloadChunk};
use pumpkin_world::cylindrical_chunk_iterator::Cylindrical;

use crate::entity::player::Player;

use super::World;

fn get_view_distance(player: &Player) -> i8 {
    player
        .config
        .view_distance
        .clamp(2, BASIC_CONFIG.view_distance as i8)
}

pub async fn player_join(world: &World, player: &mut Player) {
    let new_watched = chunk_section_from_pos(&player.entity.block_pos);
    player.watched_section = new_watched;
    let chunk_pos = player.entity.chunk_pos;
    player.client.send_packet(&CCenterChunk {
        chunk_x: chunk_pos.x.into(),
        chunk_z: chunk_pos.z.into(),
    });
    let view_distance = get_view_distance(player) as i32;
    dbg!(view_distance);
    let old_cylindrical = Cylindrical::new(
        Vector2::new(player.watched_section.x, player.watched_section.z),
        view_distance,
    );
    let new_cylindrical = Cylindrical::new(Vector2::new(chunk_pos.x, chunk_pos.z), view_distance);
    let mut loading_chunks = Vec::new();
    Cylindrical::for_each_changed_chunk(
        old_cylindrical,
        new_cylindrical,
        |chunk_pos| {
            loading_chunks.push(chunk_pos);
        },
        |chunk_pos| {
            player
                .client
                .send_packet(&CUnloadChunk::new(chunk_pos.x, chunk_pos.z));
        },
        true,
    );
    if !loading_chunks.is_empty() {
        world
            .spawn_world_chunks(&mut player.client, loading_chunks, view_distance)
            .await;
    }
}

pub async fn update_position(world: &World, player: &mut Player) {
    let current_watched = player.watched_section;
    let new_watched = chunk_section_from_pos(&player.entity.block_pos);
    if current_watched != new_watched {
        let chunk_pos = player.entity.chunk_pos;
        player.client.send_packet(&CCenterChunk {
            chunk_x: chunk_pos.x.into(),
            chunk_z: chunk_pos.z.into(),
        });

        let view_distance = get_view_distance(player) as i32;
        let old_cylindrical = Cylindrical::new(
            Vector2::new(player.watched_section.x, player.watched_section.z),
            view_distance,
        );
        let new_cylindrical =
            Cylindrical::new(Vector2::new(chunk_pos.x, chunk_pos.z), view_distance);
        player.watched_section = new_watched;
        let mut loading_chunks = Vec::new();
        Cylindrical::for_each_changed_chunk(
            old_cylindrical,
            new_cylindrical,
            |chunk_pos| {
                loading_chunks.push(chunk_pos);
            },
            |chunk_pos| {
                player
                    .client
                    .send_packet(&CUnloadChunk::new(chunk_pos.x, chunk_pos.z));
            },
            false,
        );
        if !loading_chunks.is_empty() {
            world
                .spawn_world_chunks(&mut player.client, loading_chunks, view_distance)
                .await;
        }
    }
}

fn chunk_section_from_pos(block_pos: &WorldPosition) -> Vector3<i32> {
    let block_pos = block_pos.0;
    Vector3::new(
        get_section_cord(block_pos.x),
        get_section_cord(block_pos.y),
        get_section_cord(block_pos.z),
    )
}
