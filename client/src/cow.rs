// Cow entity rendering.
// Builds a 3D cow model from the Minecraft cow texture (entity/cow/cow.png, 64x32).
// Model parts: body, head, 4 legs, 2 horns.
// All geometry is in world space, transformed per entity each frame.

use glam::{Mat4, Vec3, Quat};
use crate::mesh::BlockVertex;

// Cow texture atlas size
const TW: f32 = 64.0;
const TH: f32 = 32.0;

/// UV helper: converts pixel coords to 0..1.
fn uv(px: f32, py: f32) -> [f32; 2] {
    [px / TW, py / TH]
}

/// A single UV region [u0, v0, u1, v1].
fn region(px: f32, py: f32, pw: f32, ph: f32) -> [f32; 4] {
    [px / TW, py / TH, (px + pw) / TW, (py + ph) / TH]
}

/// Add a box (6 faces) to the mesh.
/// `min`/`max` are in local model space.
/// Each face uses the corresponding UV region.
fn add_box(
    verts: &mut Vec<BlockVertex>,
    idxs: &mut Vec<u32>,
    min: [f32; 3],
    max: [f32; 3],
    // UV regions per face: [right, left, top, bottom, front, back]
    uvs: [[f32; 4]; 6],
    transform: &Mat4,
) {
    let [x0, y0, z0] = min;
    let [x1, y1, z1] = max;

    // 6 faces: right(+X), left(-X), top(+Y), bottom(-Y), front(+Z), back(-Z)
    let faces: [([[f32; 3]; 4], [f32; 3]); 6] = [
        // right +X
        ([[x1,y0,z1],[x1,y0,z0],[x1,y1,z0],[x1,y1,z1]], [1.0, 0.0, 0.0]),
        // left -X
        ([[x0,y0,z0],[x0,y0,z1],[x0,y1,z1],[x0,y1,z0]], [-1.0, 0.0, 0.0]),
        // top +Y
        ([[x0,y1,z1],[x1,y1,z1],[x1,y1,z0],[x0,y1,z0]], [0.0, 1.0, 0.0]),
        // bottom -Y
        ([[x0,y0,z0],[x1,y0,z0],[x1,y0,z1],[x0,y0,z1]], [0.0, -1.0, 0.0]),
        // front +Z
        ([[x0,y0,z1],[x1,y0,z1],[x1,y1,z1],[x0,y1,z1]], [0.0, 0.0, 1.0]),
        // back -Z
        ([[x1,y0,z0],[x0,y0,z0],[x0,y1,z0],[x1,y1,z0]], [0.0, 0.0, -1.0]),
    ];

    for (fi, (positions, normal)) in faces.iter().enumerate() {
        let uv_r = uvs[fi];
        let base = verts.len() as u32;

        let corner_uvs = [
            [uv_r[0], uv_r[3]], // bottom-left
            [uv_r[2], uv_r[3]], // bottom-right
            [uv_r[2], uv_r[1]], // top-right
            [uv_r[0], uv_r[1]], // top-left
        ];

        for i in 0..4 {
            let p = glam::Vec4::new(positions[i][0], positions[i][1], positions[i][2], 1.0);
            let tp = *transform * p;
            let n = transform.transform_vector3(Vec3::from(*normal)).normalize();
            verts.push(BlockVertex {
                position: [tp.x, tp.y, tp.z],
                tex_coords: corner_uvs[i],
                normal: n.to_array(),
            });
        }

        idxs.extend_from_slice(&[base, base+1, base+2, base, base+2, base+3]);
    }
}

/// Build a cow mesh in world space given position and yaw.
/// Returns (vertices, indices).
pub fn build_cow_mesh(pos: Vec3, yaw: f32) -> (Vec<BlockVertex>, Vec<u32>) {
    let mut verts = Vec::with_capacity(200);
    let mut idxs = Vec::with_capacity(300);

    // Cow dimensions (in blocks, Minecraft cow ~0.9 wide, 1.4 tall):
    // Body: 10x8x6 px → scale to ~0.625 x 0.5 x 0.375 blocks
    // We scale all by 1/16 (Minecraft standard)
    let s = 1.0f32 / 16.0;

    // Root transform: world position + yaw rotation
    let root = Mat4::from_translation(pos)
        * Mat4::from_quat(Quat::from_rotation_y(yaw));

    // ── Body ──────────────────────────────────────────────────────────────────
    // Minecraft body: 10x8x6, starts at pixel (18,4) in skin
    // Center at height ~0.85 (legs=0.5 + body_half=0.25)
    let body_w = 10.0 * s; // 0.625
    let body_h = 8.0 * s;  // 0.5
    let body_d = 6.0 * s;  // 0.375
    let body_y = 0.5; // bottom of body above ground
    let body_tf = root * Mat4::from_translation(Vec3::new(0.0, body_y, 0.0));

    // Body UV layout (64x32 cow texture):
    // right(6x8)@(18,4), left(6x8)@(30,4), top(10x6)@(28,0), bottom(10x6)@(38,0)
    // front(10x8)@(28,4), back(10x8)@(40,4) — approximate Minecraft layout
    add_box(&mut verts, &mut idxs,
        [-body_w/2.0, 0.0, -body_d/2.0],
        [ body_w/2.0, body_h, body_d/2.0],
        [
            region(18.0,4.0, 6.0,8.0), // right
            region(30.0,4.0, 6.0,8.0), // left
            region(28.0,0.0,10.0,6.0), // top
            region(38.0,0.0,10.0,6.0), // bottom
            region(28.0,4.0,10.0,8.0), // front
            region(40.0,4.0,10.0,8.0), // back
        ],
        &body_tf,
    );

    // ── Head ──────────────────────────────────────────────────────────────────
    // Head: 8x8x6, at front of body, slightly above
    let head_w = 8.0 * s;
    let head_h = 8.0 * s;
    let head_d = 6.0 * s;
    let head_y = body_y + body_h + head_h * 0.1; // sit on top of neck
    let head_tz = body_d / 2.0 + head_d * 0.3;   // slightly forward
    let head_tf = root * Mat4::from_translation(Vec3::new(0.0, head_y, head_tz));

    add_box(&mut verts, &mut idxs,
        [-head_w/2.0, 0.0, -head_d/2.0],
        [ head_w/2.0, head_h, head_d/2.0],
        [
            region( 0.0,0.0, 6.0,8.0), // right
            region(12.0,0.0, 6.0,8.0), // left
            region( 6.0,0.0, 8.0,6.0), // top
            region(14.0,0.0, 8.0,6.0), // bottom
            region( 6.0,6.0, 8.0,8.0), // front (face)
            region(20.0,6.0, 8.0,8.0), // back
        ],
        &head_tf,
    );

    // ── Horns (simple small boxes) ─────────────────────────────────────────────
    let horn_w = 1.0 * s;
    let horn_h = 3.0 * s;
    for hx in [-1, 1] {
        let horn_x = (hx as f32) * (head_w / 2.0 - horn_w / 2.0 - 0.5 * s);
        let horn_tf = root * Mat4::from_translation(Vec3::new(
            horn_x,
            head_y + head_h,
            head_tz,
        ));
        add_box(&mut verts, &mut idxs,
            [-horn_w/2.0, 0.0, -horn_w/2.0],
            [ horn_w/2.0, horn_h, horn_w/2.0],
            [
                region(52.0,0.0,1.0,3.0),
                region(54.0,0.0,1.0,3.0),
                region(53.0,0.0,1.0,1.0),
                region(54.0,0.0,1.0,1.0),
                region(53.0,1.0,1.0,3.0),
                region(55.0,1.0,1.0,3.0),
            ],
            &horn_tf,
        );
    }

    // ── Legs (4 legs) ─────────────────────────────────────────────────────────
    // Each leg: 4x12x4 px → 0.25 x 0.75 x 0.25 blocks
    let leg_w = 4.0 * s;
    let leg_h = 12.0 * s; // 0.75
    // Leg positions relative to body center
    let leg_offsets: [(f32, f32); 4] = [
        (-body_w/2.0 + leg_w/2.0 + 0.5*s, -body_d/2.0 + leg_w/2.0 + 0.5*s), // front-left
        ( body_w/2.0 - leg_w/2.0 - 0.5*s, -body_d/2.0 + leg_w/2.0 + 0.5*s), // front-right
        (-body_w/2.0 + leg_w/2.0 + 0.5*s,  body_d/2.0 - leg_w/2.0 - 0.5*s), // back-left
        ( body_w/2.0 - leg_w/2.0 - 0.5*s,  body_d/2.0 - leg_w/2.0 - 0.5*s), // back-right
    ];

    for (lx, lz) in leg_offsets {
        let leg_tf = root * Mat4::from_translation(Vec3::new(lx, 0.0, lz));
        add_box(&mut verts, &mut idxs,
            [-leg_w/2.0, 0.0, -leg_w/2.0],
            [ leg_w/2.0, leg_h, leg_w/2.0],
            [
                region( 0.0,16.0, 4.0,12.0), // right
                region( 8.0,16.0, 4.0,12.0), // left
                region( 4.0,16.0, 4.0, 4.0), // top
                region( 8.0,16.0, 4.0, 4.0), // bottom
                region( 4.0,20.0, 4.0,12.0), // front
                region(12.0,20.0, 4.0,12.0), // back
            ],
            &leg_tf,
        );
    }

    (verts, idxs)
}
