use super::cli::Args;
use bevy::core::FixedTimestep;
use bevy::prelude::*;
use bevy_prototype_debug_lines::*;
use rand::prelude::*;
use std::collections::HashMap;
use std::f64::consts::{E, PI};
use uuid::Uuid;

// ============ CONSTANTS ============

pub const LOGISTIC_OPINION_SCALE: f64 = -0.01;

// ============ RESOURCES ============

#[derive(Default)]
pub struct FaceDirectory {
    faces: HashMap<String, Handle<Image>>,
}

#[derive(Default)]
pub struct TransformState {
    pub transforms: HashMap<String, Transform>,
}

impl TransformState {
    fn get(&self, k: &String) -> Option<&Transform> {
        self.transforms.get(k)
    }
}

#[derive(Default, Clone)]
pub struct SpriteRegistry {
    characters: HashMap<String, Handle<Image>>,
    thumbs_up: Handle<Image>,
    thumbs_down: Handle<Image>,
    thought: Handle<Image>,
    speech: Handle<Image>,
}

impl SpriteRegistry {
    fn random_character(&self) -> String {
        self.characters
            .keys()
            .choose(&mut rand::thread_rng())
            .unwrap()
            .to_string()
    }

    fn get_character(&self, k: &String) -> Handle<Image> {
        self.characters.get(k).unwrap().clone()
    }
}

// ============ COMPONENTS ============

#[derive(Component)]
pub struct Agent;

#[derive(Component)]
pub struct ID(String);

impl ID {
    fn rand() -> Self {
        ID(Uuid::new_v4().to_string())
    }
}

#[derive(Component)]
pub struct Identity(Handle<Image>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn spritesheet_row(&self) -> usize {
        match self {
            Direction::Up => 3,
            Direction::Down => 0,
            Direction::Left => 1,
            Direction::Right => 2,
        }
    }
}

#[derive(Component)]
pub struct Body {
    pub velocity: Vec3,
}

#[derive(Component)]
pub struct Brain;

#[derive(Component)]
pub struct Personality {
    pub chattiness: usize,
}

impl Personality {
    fn random() -> Self {
        let mut rng = rand::thread_rng();
        let chattiness: usize = rng.gen_range(0..100);

        Personality { chattiness }
    }
}

#[derive(Component)]
pub struct Lifetime(Timer);

#[derive(Component)]
pub struct Voice;

#[derive(Component, Default)]
pub struct Opinions {
    people: HashMap<String, PersonalOpinion>,
    favorite_person: (String, f64),
    // locations could be used for determining whether areas are favorable to go to over long term, allowing for agents to learn where their friends tend to congregate
    // locations : HashMap<String, LocationOpinion>,
}

impl Opinions {
    fn new(owner_id: String) -> Self {
        let mut people = HashMap::<String, PersonalOpinion>::new();
        //intial self-love can eventually be determined by provided Personality, as opposed to everyone starting feeling so-so about themselves
        let opinion_of_self = PersonalOpinion::new(100.0, 100.0);
        let fav_person_tuple = (owner_id.clone(), opinion_of_self.likeability.clone());

        people.insert(owner_id, opinion_of_self);

        Opinions {
            people: people,
            favorite_person: fav_person_tuple,
        }
    }

    fn check_if_new_favorite(
        &mut self,
        candidate_opinion: &PersonalOpinion,
        candidate_id: &String,
    ) {
        let likeability_threshold = self.favorite_person.1;
        if candidate_opinion.likeability > likeability_threshold {
            self.favorite_person = (candidate_id.clone(), candidate_opinion.likeability.clone());
        }
    }

    fn get_fav_person_id(&self) -> String {
        self.favorite_person.0.clone()
    }

    /// At Presemt, this implements a random choice from a set of held opinions to an output statement
    /// future iterations should select statements based on their relevance
    /// relevance can be held in a kind of conceptual hierarchy component along with status.
    /// these could both be sorted vectors of tuples (score, topic), whose score defines a ranking on individuals.
    /// A choice from the vector should be a weighted random draw, where 0 is k times more likely than n for a vec.len() = n + 1;
    /// should define a pretty straightforward curve, can possibly do with rounding on a curve func with input set to int;
    fn generate_speakable_personal_opinion(
        &mut self,
        transform: &Transform,
        id: &ID,
        identity: &Identity,
    ) -> SpokenEvent {
        let person = self.people.keys().choose(&mut rand::thread_rng());

        let topic: String = person.map(|s| s.to_string()).unwrap_or(id.0.clone());

        let opinion: PersonalOpinion = self
            .people
            .get(&topic)
            .expect("Could not get opinion")
            .clone();

        let event = SpokenEvent {
            author: id.0.clone(),
            origin: transform.translation,
            distance: 150.0,
            identity: identity.0.clone(),
            opinion: Some((topic.clone(), opinion.clone())),
        };

        event
        //let opinion = opinions.people.get(person);
    }
}

#[derive(Component, Clone)]
pub struct PersonalOpinion {
    trust_seed: f64,
    pub trust: f64,
    likeability_seed: f64,
    pub likeability: f64,
}

impl PersonalOpinion {
    /// Simple tethered adjust, can grow more specific, situational, and complex as sim develops
    /// trust is not really integrated yet, as it increases complexity significantly,
    /// and is dependant on not-yet-implemented contradiction detection.
    fn adjust_trust(&mut self, modifier_value: f64) {
        self.trust = self.trust_seed + modifier_value;
        self.likeability = self.likeability_seed + (0.5 * modifier_value);
        self.propegate_output_values();
    }

    /// Simple tethered adjust, can grow more specific, situational, and complex as sim develops.
    fn adjust_likeability(&mut self, modifier_value: f64) {
        self.trust = self.trust_seed + (0.5 * modifier_value);
        self.likeability = self.likeability_seed + modifier_value;
        self.propegate_output_values();
    }

    /// Logistic function provides high stability around extreme affection, and extreme dislike, but more variability while on the fence.
    /// The degree to which variabiliy happens around origin is determined by const LOGISTIC_OPINION_SCALE.
    fn propegate_output_values(&mut self) {
        self.trust = (200.0 / (1.0 + E.powf(LOGISTIC_OPINION_SCALE * self.trust_seed))) - 100.0;
        self.likeability =
            (200.00 / (1.0 + E.powf(LOGISTIC_OPINION_SCALE * self.likeability_seed))) - 100.0;
    }

    fn new(init_trust: f64, init_likeability: f64) -> Self {
        let mut output_opinion = PersonalOpinion {
            trust: 0.0,
            likeability: 0.0,
            trust_seed: init_trust,
            likeability_seed: init_likeability,
        };
        output_opinion.propegate_output_values();

        output_opinion
    }
}

// ============ EVENTS ============

struct SpokenEvent {
    author: String,
    origin: Vec3,
    distance: f32,
    identity: Handle<Image>,
    opinion: Option<(String, PersonalOpinion)>,
}
// ============ SYSTEM LABELS ============

#[derive(SystemLabel, Clone, Hash, Debug, Eq, PartialEq)]
enum StartupLabels {
    LoadSprites,
    PopulateSim,
    MakeRivals,
}

// ============ PLUGIN ============
pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpokenEvent>()
            .init_resource::<TransformState>()
            .init_resource::<FaceDirectory>()
            .init_resource::<SpriteRegistry>()
            .add_startup_system(load_sprites_startup.label(StartupLabels::LoadSprites))
            .add_startup_system(setup_startup)
            .add_startup_system(
                populate_sim_startup
                    .label(StartupLabels::PopulateSim)
                    .after(StartupLabels::LoadSprites),
            )
            .add_startup_system(
                make_rivals_startup
                    .label(StartupLabels::MakeRivals)
                    .after(StartupLabels::LoadSprites),
            )
            .add_startup_system(report_agent_transform_system.after(StartupLabels::PopulateSim))
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(0.1))
                    .with_system(animate_sprite_system)
                    .with_system(executive_functioning_system),
            )
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(0.2))
                    .with_system(say_system),
            )
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(0.01))
                    .with_system(boundaries_system)
                    .with_system(physics_system)
                    .with_system(report_agent_transform_system),
            )
            .add_system(direct_sprite_system)
            .add_system(thought_system)
            .add_system(lifetime_despawn_system);
    }
}

// ============ STARTUP SYSTEMS ============

fn setup_startup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
}

fn populate_sim_startup(
    mut commands: Commands,
    sprites: Res<SpriteRegistry>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut face_directory: ResMut<FaceDirectory>,
    args: Res<Args>,
) {
    info!("Populating simulation");
    for _ in 0..args.population {
        make_rand_character(
            &mut commands,
            &sprites,
            &mut texture_atlases,
            &mut face_directory,
        )
    }
}

fn load_sprites_startup(
    asset_server: Res<AssetServer>,
    mut sprite_registry: ResMut<SpriteRegistry>,
) {
    info!("Loading population sprites");
    let sprites = vec![
        "lady.png",
        "baldguy.png",
        "coollady.png",
        "princessleia.png",
        "blondedude.png",
        "hatguy.png",
        "redhead.png",
        "jacketguy.png",
    ];

    let sprites_map: HashMap<String, Handle<Image>> = sprites
        .iter()
        .map(|sprite| {
            let sprite_handle: Handle<Image> = asset_server.load(sprite.to_owned());
            (sprite.to_string(), sprite_handle)
        })
        .collect();

    sprite_registry.characters = sprites_map;

    info!("Loading bubble sprites");
    sprite_registry.thought = asset_server.load("thoughtbubble.png");
    sprite_registry.speech = asset_server.load("textbubble.png");

    info!("Loading sentiment sprites");
    sprite_registry.thumbs_up = asset_server.load("good_thumbs_up.png");
    sprite_registry.thumbs_down = asset_server.load("bad_thumbs_down.png");
}

fn make_rivals_startup(mut query: Query<&mut Opinions>, face_directory: Res<FaceDirectory>) {
    for mut opinions in query.iter_mut() {
        let person = face_directory
            .faces
            .keys()
            .choose(&mut rand::thread_rng())
            .unwrap();
        let person2 = face_directory
            .faces
            .keys()
            .choose(&mut rand::thread_rng())
            .unwrap();
        opinions
            .people
            .insert(person.clone(), PersonalOpinion::new(-100.0, -100.0));
        opinions
            .people
            .insert(person2.clone(), PersonalOpinion::new(-100.0, -100.0));
    }
}

// ============ SYSTEMS ============

fn animate_sprite_system(mut query: Query<(&mut TextureAtlasSprite, &Body, &Direction)>) {
    for (mut sprite, body, direction) in query.iter_mut() {
        if body.velocity.x.abs() < 0.01 && body.velocity.y.abs() < 0.01 {
            continue;
        }
        let row = direction.spritesheet_row();
        sprite.index = (row * 3 as usize) + (sprite.index + 1) % 3;
    }
}

fn make_rand_character(
    commands: &mut Commands,
    sprites: &Res<SpriteRegistry>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
    face_directory: &mut ResMut<FaceDirectory>,
) {
    let sprite = sprites.random_character();
    let sprite_handle: Handle<Image> = sprites.get_character(&sprite);

    let texture_atlas = TextureAtlas::from_grid(sprite_handle.clone(), Vec2::new(52.0, 72.0), 3, 4);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let id = ID::rand();
    let num_name = id.0.clone();

    face_directory
        .faces
        .insert(num_name.clone(), sprite_handle.clone());

    let mut rng = rand::thread_rng();
    let initial_location: (f32, f32) = (rng.gen_range(-200.0..200.0), rng.gen_range(-200.0..200.0));

    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform {
                translation: Vec3::new(initial_location.0, initial_location.1, 1.0),
                scale: Vec3::new(1.0, 1.0, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(id)
        .insert(Identity(sprite_handle.clone()))
        .insert(Body {
            velocity: 0.0 * Vec3::new(0.5, -0.5, 0.0).normalize(),
        })
        .insert(Direction::Right)
        .insert(Voice)
        .insert(Personality::random())
        .insert(Opinions::new(num_name.clone()))
        .insert(Brain);
}

fn executive_functioning_system(
    mut query: Query<(&mut Body, &Opinions, &Transform), (With<Brain>, With<Direction>)>,
    transform_state: Res<TransformState>,
) {
    let mut rng = rand::thread_rng();

    for (mut body, opinions, transform) in query.iter_mut() {
        let should_turn: usize = rng.gen_range(0..100);
        let should_random: usize = rng.gen_range(0..100);

        if should_turn <= 10 {
            let actor_translation: Vec3 = transform.translation;
            let favorite_person_id: String = opinions.get_fav_person_id();

            if let Some(target_transform) = transform_state.get(&favorite_person_id) {
                let target_translation = target_transform.translation;
                let non_normal_vec = target_translation - actor_translation.clone();

                if non_normal_vec == Vec3::ZERO || should_random == 69 {
                    let k: f64 = rng.gen_range(0.0..2000.0);
                    let rads = k / 1000.0 * PI;
                    body.velocity.x = rads.cos() as f32;
                    body.velocity.y = rads.sin() as f32;
                } else {
                    body.velocity = non_normal_vec.normalize();
                }
            }
        }
    }
}

fn direct_sprite_system(
    mut query: Query<(&Body, &mut Direction), (Changed<Body>, With<Transform>)>,
) {
    for (body, mut direction) in query.iter_mut() {
        let x = body.velocity.x;
        let y = body.velocity.y;

        *direction = if x.abs() > y.abs() {
            if x < 0.0 {
                Direction::Left
            } else {
                Direction::Right
            }
        } else {
            if y < 0.0 {
                Direction::Down
            } else {
                Direction::Up
            }
        };

        debug!("vel -> dir => {:?} -> {:?}", body.velocity, direction);
    }
}

fn physics_system(mut query: Query<(&Body, &mut Transform)>) {
    for (body, mut transform) in query.iter_mut() {
        transform.translation.x += body.velocity.x;
        transform.translation.y += body.velocity.y;
    }
}

fn report_agent_transform_system(
    mut query: Query<(&Transform, &Body, &ID), With<Brain>>,
    mut transform_global_state: ResMut<TransformState>,
) {
    for (transform, body, id) in query.iter_mut() {
        transform_global_state
            .transforms
            .entry(id.0.clone())
            .or_insert(transform.clone());
    }
}

fn boundaries_system(mut query: Query<(&mut Body, &Transform), With<Direction>>) {
    for (mut body, transform) in query.iter_mut() {
        //these should be on window size, not manually hard-coded
        let bounds = (
            transform.translation.y > 400.0,
            transform.translation.y < -400.0,
            transform.translation.x > 700.0,
            transform.translation.x < -700.0,
        );

        match bounds {
            (true, _, _, _) => {
                body.velocity.x = 0.0;
                body.velocity.y = -1.0;
            }
            (_, true, _, _) => {
                body.velocity.x = 0.0;
                body.velocity.y = 1.0;
            }
            (_, _, true, _) => {
                body.velocity.x = -1.0;
                body.velocity.y = 0.0;
            }
            (_, _, _, true) => {
                body.velocity.x = 1.0;
                body.velocity.y = 0.0;
            }
            _ => {}
        }
    }
}

fn say_system(
    mut commands: Commands,
    mut spoken_events: EventWriter<SpokenEvent>,
    sprites: Res<SpriteRegistry>,
    mut query: Query<(
        Entity,
        &ID,
        &Identity,
        &Transform,
        &Personality,
        &Voice,
        &mut Opinions,
    )>,
) {
    let mut rng = rand::thread_rng();

    for (entity, id, identity, transform, personality, _, mut opinions) in query.iter_mut() {
        let should_think: usize = rng.gen_range(0..10000);

        if should_think <= personality.chattiness {
            let texture = sprites.speech.clone();

            let child = commands
                .spawn_bundle(SpriteBundle {
                    texture: texture.clone(),
                    transform: Transform {
                        translation: Vec3::new(40.0, 45.0, 1.0),
                        scale: Vec3::new(1.0, 1.0, 1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Lifetime(Timer::from_seconds(2.0, true)))
                .id();

            commands.entity(entity).push_children(&[child]);

            let event = opinions.generate_speakable_personal_opinion(&transform, &id, &identity);
            spoken_events.send(event);
        }
    }
}

fn thought_system(
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut lines: ResMut<DebugLines>,
    mut spoken_events: EventReader<SpokenEvent>,
    mut query: Query<(Entity, &Transform, &ID, &Brain, &mut Opinions)>,
    sprites: Res<SpriteRegistry>,
    face_directory: Res<FaceDirectory>,
) {
    for spoken_event in spoken_events.iter() {
        for (entity, transform, id, _, mut opinions) in query.iter_mut() {
            if spoken_event.origin.distance(transform.translation) <= spoken_event.distance
                && id.0 != spoken_event.author
            {
                let texture = sprites.thought.clone();

                let bubble = commands
                    .spawn_bundle(SpriteBundle {
                        texture: texture.clone(),
                        transform: Transform {
                            translation: Vec3::new(40.0, 45.0, 2.0),
                            scale: Vec3::new(1.0, 1.0, 1.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(Lifetime(Timer::from_seconds(2.0, true)))
                    .id();

                if let Some((subject_id, personal_opinion)) = &spoken_event.opinion {
                    let subject: String = subject_id.clone();
                    let transmitted_opinion: PersonalOpinion = personal_opinion.clone();

                    process_heard_opinion(
                        &mut opinions,
                        &spoken_event.author,
                        &subject,
                        &transmitted_opinion,
                    );

                    let face_texture = face_directory.faces.get(&subject);

                    if let Some(handle) = face_texture {
                        let face_handle = handle;
                        let face_atlas = TextureAtlas::from_grid(
                            face_handle.clone(),
                            Vec2::new(45.0, 45.0),
                            1,
                            1,
                        );
                        let face_atlas_handle = texture_atlases.add(face_atlas);
                        let gossip_subject_face = commands
                            .spawn_bundle(SpriteSheetBundle {
                                texture_atlas: face_atlas_handle,
                                transform: Transform {
                                    translation: Vec3::new(5.0, 0.0, 4.0),
                                    scale: Vec3::new(0.7, 0.7, 1.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .id();
                        commands
                            .entity(bubble)
                            .push_children(&[gossip_subject_face]);
                    }

                    let mut start_color = Color::BLUE;
                    let mut end_color = Color::GREEN;
                    let mut value_icon_texture = sprites.thumbs_down.clone();

                    if transmitted_opinion.likeability > 0.0 {
                        start_color = Color::RED;
                        end_color = Color::ORANGE;
                        value_icon_texture = sprites.thumbs_up.clone();
                    }

                    let judgement = commands
                        .spawn_bundle(SpriteBundle {
                            texture: value_icon_texture,
                            transform: Transform {
                                translation: Vec3::new(-15.0, 3.0, 2.0),
                                scale: Vec3::new(0.5, 0.5, 1.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .id();

                    commands.entity(bubble).push_children(&[judgement]);

                    lines.line_gradient(
                        spoken_event.origin,
                        transform.translation,
                        0.3,
                        start_color,
                        end_color,
                    );
                }

                // Rendering head of the source model
                //asset_server Load funcs should be in setup only, as they access filesystem.
                let sprite_handle = &spoken_event.identity;
                let texture_atlas =
                    TextureAtlas::from_grid(sprite_handle.clone(), Vec2::new(45.0, 45.0), 1, 1);
                let texture_atlas_handle = texture_atlases.add(texture_atlas);
                let head = commands
                    .spawn_bundle(SpriteSheetBundle {
                        texture_atlas: texture_atlas_handle,
                        transform: Transform {
                            translation: Vec3::new(20.0, 20.0, 4.0),
                            scale: Vec3::new(0.7, 0.7, 1.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .id();

                commands.entity(bubble).push_children(&[head]);
                commands.entity(entity).push_children(&[bubble]);
            }
        }
    }
}

fn lifetime_despawn_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut auto_remove) in query.iter_mut() {
        if auto_remove.0.tick(time.delta()).finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// ============ SUBSYSTEMS ============

fn process_heard_opinion(
    listener_opinions: &mut Opinions,
    speaker_id: &String,
    subject_id: &String,
    transmitted_opinion: &PersonalOpinion,
) {
    // let mut value: f64 = 0.0;
    let mut held_opinion_of_speaker: f64 = 0.0;
    let mut held_opinion_of_subject: f64 = 0.0;
    let transmitted_value_judgement: f64 = transmitted_opinion.likeability.clone();

    match (
        listener_opinions.people.get(speaker_id),
        listener_opinions.people.get(subject_id),
    ) {
        (Some(held_speaker_opinion), Some(held_subject_opinion)) => {
            held_opinion_of_speaker = held_speaker_opinion.likeability.clone();
            held_opinion_of_subject = held_subject_opinion.likeability.clone();
        }
        (Some(held_speaker_opinion), None) => {
            held_opinion_of_speaker = held_speaker_opinion.likeability.clone();
            listener_opinions
                .people
                .insert(subject_id.clone(), get_initial_impression());
        }
        (None, Some(held_subject_opinion)) => {
            held_opinion_of_subject = held_subject_opinion.likeability.clone();
            listener_opinions
                .people
                .insert(subject_id.clone(), get_initial_impression());
        }
        (None, None) => {
            listener_opinions
                .people
                .insert(speaker_id.clone(), get_initial_impression());
            listener_opinions
                .people
                .insert(subject_id.clone(), get_initial_impression());
        }
    }

    if held_opinion_of_speaker > 0.0 {
        let weight = 100.0 - held_opinion_of_speaker;
        let op: &mut PersonalOpinion = listener_opinions
            .people
            .get_mut(&subject_id.clone())
            .unwrap();
        op.adjust_likeability(weight * transmitted_value_judgement);
    } else {
        let weight = -100.0 - held_opinion_of_speaker;
        let op: &mut PersonalOpinion = listener_opinions.people.get_mut(subject_id).unwrap();
        op.adjust_likeability(weight * transmitted_value_judgement);
    }

    let subject_opinion = listener_opinions
        .people
        .get(&subject_id.clone())
        .unwrap()
        .clone();
    listener_opinions.check_if_new_favorite(&subject_opinion, &subject_id);

    fn get_initial_impression() -> PersonalOpinion {
        let mut rng = rand::thread_rng();
        let value: f64 = rng.gen_range(-50.0..50.0);

        PersonalOpinion::new(value.clone(), value.clone())
    }
}
