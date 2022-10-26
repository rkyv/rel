mod data;

use ::criterion::black_box;
use ::mischief::{GhostRef, In, Region, Slot, StaticToken};
use ::munge::munge;
use ::ptr_meta::Pointee;
use ::rand::Rng;
use ::rel_alloc::{alloc::RelAllocator, EmplaceIn, RelString, RelVec};
use ::rel_core::{
    option::RelOption,
    rel_tuple::{RelTuple2, RelTuple3},
    Emplace,
    EmplaceExt,
    Move,
    Portable,
    F32,
    F64,
    I32,
    I64,
    U16,
    U32,
};
use ::rel_slab_allocator::{RelSlabAllocator, SlabAllocator};
use ::rel_util::Align16;
use ::situ::{alloc::RawRegionalAllocator, DropRaw};

use crate::{from_data::FromData, gen::generate_vec};

#[derive(DropRaw, Move, Portable)]
#[repr(u8)]
pub enum RelGameType {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

unsafe impl<R: Region> Emplace<RelGameType, R> for &'_ data::GameType {
    fn emplaced_meta(&self) -> <RelGameType as Pointee>::Metadata {}

    unsafe fn emplace_unsized_unchecked(
        self,
        out: In<Slot<'_, RelGameType>, R>,
    ) {
        let value = match self {
            data::GameType::Survival => RelGameType::Survival,
            data::GameType::Creative => RelGameType::Creative,
            data::GameType::Adventure => RelGameType::Adventure,
            data::GameType::Spectator => RelGameType::Spectator,
        };
        In::into_inner(out).write(value);
    }
}

#[derive(DropRaw, Move, Portable)]
#[repr(C)]
pub struct RelItem<A: RawRegionalAllocator> {
    pub count: i8,
    pub slot: u8,
    pub id: RelString<A>,
}

unsafe impl<A, R> Emplace<RelItem<A>, R::Region> for FromData<'_, R, data::Item>
where
    A: DropRaw + Move<R::Region> + RawRegionalAllocator<Region = R::Region>,
    R: Clone + RelAllocator<A>,
{
    fn emplaced_meta(&self) -> <RelItem<A> as Pointee>::Metadata {}

    unsafe fn emplace_unsized_unchecked(
        self,
        out: In<Slot<'_, RelItem<A>>, R::Region>,
    ) {
        use ::rel_alloc::string;

        munge!(
            let RelItem {
                count,
                slot,
                id,
            } = out;
        );

        self.data.count.emplace(count);
        self.data.slot.emplace(slot);
        string::Clone(self.alloc.clone(), &self.data.id).emplace(id);
    }
}

#[derive(DropRaw, Move, Portable)]
#[repr(C)]
pub struct RelAbilities {
    pub walk_speed: F32,
    pub fly_speed: F32,
    pub may_fly: bool,
    pub flying: bool,
    pub invulnerable: bool,
    pub may_build: bool,
    pub instabuild: bool,
}

unsafe impl<R: Region> Emplace<RelAbilities, R> for &'_ data::Abilities {
    fn emplaced_meta(&self) -> <RelAbilities as Pointee>::Metadata {}

    unsafe fn emplace_unsized_unchecked(
        self,
        out: In<Slot<'_, RelAbilities>, R>,
    ) {
        munge!(
            let RelAbilities {
                walk_speed,
                fly_speed,
                may_fly,
                flying,
                invulnerable,
                may_build,
                instabuild,
            } = out;
        );

        self.walk_speed.emplace(walk_speed);
        self.fly_speed.emplace(fly_speed);
        self.may_fly.emplace(may_fly);
        self.flying.emplace(flying);
        self.invulnerable.emplace(invulnerable);
        self.may_build.emplace(may_build);
        self.instabuild.emplace(instabuild);
    }
}

#[derive(DropRaw, Move, Portable)]
#[repr(C)]
pub struct RelEntity<A: RawRegionalAllocator> {
    pub id: RelString<A>,
    pub pos: RelTuple3<F64, F64, F64>,
    pub motion: RelTuple3<F64, F64, F64>,
    pub rotation: RelTuple2<F32, F32>,
    pub fall_distance: F32,
    pub fire: U16,
    pub air: U16,
    pub on_ground: bool,
    pub no_gravity: bool,
    pub invulnerable: bool,
    pub portal_cooldown: I32,
    pub uuid: [U32; 4],
    pub custom_name: RelOption<RelString<A>>,
    pub custom_name_visible: bool,
    pub silent: bool,
    pub glowing: bool,
}

unsafe impl<A, R> Emplace<RelEntity<A>, R::Region>
    for FromData<'_, R, data::Entity>
where
    A: DropRaw + Move<R::Region> + RawRegionalAllocator<Region = R::Region>,
    R: Clone + RelAllocator<A>,
{
    fn emplaced_meta(&self) -> <RelEntity<A> as Pointee>::Metadata {}

    unsafe fn emplace_unsized_unchecked(
        self,
        out: In<Slot<'_, RelEntity<A>>, R::Region>,
    ) {
        use ::rel_alloc::string;

        munge!(
            let RelEntity {
                id,
                pos,
                motion,
                rotation,
                fall_distance,
                fire,
                air,
                on_ground,
                no_gravity,
                invulnerable,
                portal_cooldown,
                uuid,
                custom_name,
                custom_name_visible,
                silent,
                glowing,
            } = out;
        );

        string::Clone(self.alloc.clone(), &self.data.id).emplace(id);
        self.data.pos.emplace(pos);
        self.data.motion.emplace(motion);
        self.data.rotation.emplace(rotation);
        self.data.fall_distance.emplace(fall_distance);
        self.data.fire.emplace(fire);
        self.data.air.emplace(air);
        self.data.on_ground.emplace(on_ground);
        self.data.no_gravity.emplace(no_gravity);
        self.data.invulnerable.emplace(invulnerable);
        self.data.portal_cooldown.emplace(portal_cooldown);
        self.data.uuid.emplace(uuid);
        self.data
            .custom_name
            .as_ref()
            .map(|s| string::Clone(self.alloc.clone(), s))
            .emplace(custom_name);
        self.data.custom_name_visible.emplace(custom_name_visible);
        self.data.silent.emplace(silent);
        self.data.glowing.emplace(glowing);
    }
}

#[derive(DropRaw, Move, Portable)]
#[repr(C)]
pub struct RelRecipeBook<A: RawRegionalAllocator> {
    pub recipes: RelVec<RelString<A>, A>,
    pub to_be_displayed: RelVec<RelString<A>, A>,
    pub is_filtering_craftable: bool,
    pub is_gui_open: bool,
    pub is_furnace_filtering_craftable: bool,
    pub is_furnace_gui_open: bool,
    pub is_blasting_furnace_filtering_craftable: bool,
    pub is_blasting_furnace_gui_open: bool,
    pub is_smoker_filtering_craftable: bool,
    pub is_smoker_gui_open: bool,
}

unsafe impl<A, R> Emplace<RelRecipeBook<A>, R::Region>
    for FromData<'_, R, data::RecipeBook>
where
    A: DropRaw + Move<R::Region> + RawRegionalAllocator<Region = R::Region>,
    R: Clone + RelAllocator<A>,
{
    fn emplaced_meta(&self) -> <RelRecipeBook<A> as Pointee>::Metadata {}

    unsafe fn emplace_unsized_unchecked(
        self,
        out: In<Slot<'_, RelRecipeBook<A>>, R::Region>,
    ) {
        use ::rel_alloc::{string, vec};

        munge!(
            let RelRecipeBook {
                recipes,
                to_be_displayed,
                is_filtering_craftable,
                is_gui_open,
                is_furnace_filtering_craftable,
                is_furnace_gui_open,
                is_blasting_furnace_filtering_craftable,
                is_blasting_furnace_gui_open,
                is_smoker_filtering_craftable,
                is_smoker_gui_open,
            } = out;
        );

        let recipes =
            vec::WithCapacity(self.alloc.clone(), self.data.recipes.len())
                .emplace_mut(recipes);
        RelVec::extend(
            In::into_inner(recipes),
            self.data
                .recipes
                .iter()
                .map(|data| string::Clone(self.alloc.clone(), data)),
        );

        let to_be_displayed = vec::WithCapacity(
            self.alloc.clone(),
            self.data.to_be_displayed.len(),
        )
        .emplace_mut(to_be_displayed);
        RelVec::extend(
            In::into_inner(to_be_displayed),
            self.data
                .to_be_displayed
                .iter()
                .map(|data| string::Clone(self.alloc.clone(), data)),
        );

        self.data
            .is_filtering_craftable
            .emplace(is_filtering_craftable);
        self.data.is_gui_open.emplace(is_gui_open);
        self.data
            .is_furnace_filtering_craftable
            .emplace(is_furnace_filtering_craftable);
        self.data.is_furnace_gui_open.emplace(is_furnace_gui_open);
        self.data
            .is_blasting_furnace_filtering_craftable
            .emplace(is_blasting_furnace_filtering_craftable);
        self.data
            .is_blasting_furnace_gui_open
            .emplace(is_blasting_furnace_gui_open);
        self.data
            .is_smoker_filtering_craftable
            .emplace(is_smoker_filtering_craftable);
        self.data.is_smoker_gui_open.emplace(is_smoker_gui_open);
    }
}

#[derive(DropRaw, Move, Portable)]
#[repr(C)]
pub struct RelPlayer<A: RawRegionalAllocator> {
    pub game_type: RelGameType,
    pub previous_game_type: RelGameType,
    pub score: I64,
    pub dimension: RelString<A>,
    pub selected_item_slot: U32,
    pub selected_item: RelItem<A>,
    pub spawn_dimension: RelOption<RelString<A>>,
    pub spawn_x: I64,
    pub spawn_y: I64,
    pub spawn_z: I64,
    pub spawn_forced: RelOption<bool>,
    pub sleep_timer: U16,
    pub food_exhaustion_level: F32,
    pub food_saturation_level: F32,
    pub food_tick_timer: U32,
    pub xp_level: U32,
    pub xp_p: F32,
    pub xp_total: I32,
    pub xp_seed: I32,
    pub inventory: RelVec<RelItem<A>, A>,
    pub ender_items: RelVec<RelItem<A>, A>,
    pub abilities: RelAbilities,
    pub entered_nether_position: RelOption<RelTuple3<F64, F64, F64>>,
    pub root_vehicle: RelOption<RelTuple2<[U32; 4], RelEntity<A>>>,
    pub shoulder_entity_left: RelOption<RelEntity<A>>,
    pub shoulder_entity_right: RelOption<RelEntity<A>>,
    pub seen_credits: bool,
    pub recipe_book: RelRecipeBook<A>,
}

unsafe impl<A, R> Emplace<RelPlayer<A>, R::Region>
    for FromData<'_, R, data::Player>
where
    A: DropRaw + Move<R::Region> + RawRegionalAllocator<Region = R::Region>,
    R: Clone + RelAllocator<A>,
{
    fn emplaced_meta(&self) -> <RelPlayer<A> as Pointee>::Metadata {}

    unsafe fn emplace_unsized_unchecked(
        self,
        out: In<Slot<'_, RelPlayer<A>>, R::Region>,
    ) {
        use ::rel_alloc::{string, vec};

        munge!(
            let RelPlayer {
                game_type,
                previous_game_type,
                score,
                dimension,
                selected_item_slot,
                selected_item,
                spawn_dimension,
                spawn_x,
                spawn_y,
                spawn_z,
                spawn_forced,
                sleep_timer,
                food_exhaustion_level,
                food_saturation_level,
                food_tick_timer,
                xp_level,
                xp_p,
                xp_total,
                xp_seed,
                inventory,
                ender_items,
                abilities,
                entered_nether_position,
                root_vehicle,
                shoulder_entity_left,
                shoulder_entity_right,
                seen_credits,
                recipe_book,
            } = out;
        );

        self.data.game_type.emplace(game_type);
        self.data.previous_game_type.emplace(previous_game_type);
        self.data.score.emplace(score);
        string::Clone(self.alloc.clone(), &self.data.dimension)
            .emplace(dimension);
        self.data.selected_item_slot.emplace(selected_item_slot);
        FromData {
            alloc: self.alloc.clone(),
            data: &self.data.selected_item,
        }
        .emplace(selected_item);
        self.data
            .spawn_dimension
            .as_ref()
            .map(|d| string::Clone(self.alloc.clone(), d))
            .emplace(spawn_dimension);
        self.data.spawn_x.emplace(spawn_x);
        self.data.spawn_y.emplace(spawn_y);
        self.data.spawn_z.emplace(spawn_z);
        self.data.spawn_forced.emplace(spawn_forced);
        self.data.sleep_timer.emplace(sleep_timer);
        self.data
            .food_exhaustion_level
            .emplace(food_exhaustion_level);
        self.data
            .food_saturation_level
            .emplace(food_saturation_level);
        self.data.food_tick_timer.emplace(food_tick_timer);
        self.data.xp_level.emplace(xp_level);
        self.data.xp_p.emplace(xp_p);
        self.data.xp_total.emplace(xp_total);
        self.data.xp_seed.emplace(xp_seed);

        let inventory =
            vec::WithCapacity(self.alloc.clone(), self.data.inventory.len())
                .emplace_mut(inventory);
        RelVec::extend(
            In::into_inner(inventory),
            self.data.inventory.iter().map(|data| FromData {
                alloc: self.alloc.clone(),
                data,
            }),
        );

        let ender_items =
            vec::WithCapacity(self.alloc.clone(), self.data.ender_items.len())
                .emplace_mut(ender_items);
        RelVec::extend(
            In::into_inner(ender_items),
            self.data.ender_items.iter().map(|data| FromData {
                alloc: self.alloc.clone(),
                data,
            }),
        );

        self.data.abilities.emplace(abilities);
        self.data
            .entered_nether_position
            .emplace(entered_nether_position);
        self.data
            .root_vehicle
            .as_ref()
            .map(|(attach, entity)| {
                (
                    *attach,
                    FromData {
                        alloc: self.alloc.clone(),
                        data: entity,
                    },
                )
            })
            .emplace(root_vehicle);
        self.data
            .shoulder_entity_left
            .as_ref()
            .map(|data| FromData {
                alloc: self.alloc.clone(),
                data,
            })
            .emplace(shoulder_entity_left);
        self.data
            .shoulder_entity_right
            .as_ref()
            .map(|data| FromData {
                alloc: self.alloc.clone(),
                data,
            })
            .emplace(shoulder_entity_right);
        self.data.seen_credits.emplace(seen_credits);
        FromData {
            alloc: self.alloc.clone(),
            data: &self.data.recipe_book,
        }
        .emplace(recipe_book);
    }
}

#[derive(DropRaw, Move, Portable)]
#[repr(C)]
pub struct RelSaveData<A: RawRegionalAllocator> {
    pub players: RelVec<RelPlayer<A>, A>,
}

unsafe impl<A, R> Emplace<RelSaveData<A>, R::Region>
    for FromData<'_, R, data::SaveData>
where
    A: DropRaw + Move<R::Region> + RawRegionalAllocator<Region = R::Region>,
    R: Clone + RelAllocator<A>,
{
    fn emplaced_meta(&self) -> <RelSaveData<A> as Pointee>::Metadata {}

    unsafe fn emplace_unsized_unchecked(
        self,
        out: In<Slot<'_, RelSaveData<A>>, R::Region>,
    ) {
        use ::rel_alloc::vec;

        munge!(let RelSaveData { players } = out);

        let players =
            vec::WithCapacity(self.alloc.clone(), self.data.players.len())
                .emplace_mut(players);
        RelVec::extend(
            In::into_inner(players),
            self.data.players.iter().map(|data| FromData {
                alloc: self.alloc.clone(),
                data,
            }),
        );
    }
}

fn populate_buffer(data: &data::SaveData, buffer: Slot<'_, [u8]>) -> usize {
    StaticToken::acquire(|mut token| {
        let alloc =
            SlabAllocator::<_>::try_new_in(buffer, GhostRef::leak(&mut token))
                .unwrap();

        let save_data = FromData { alloc, data }
            .emplace_in::<RelSaveData<RelSlabAllocator<_>>>(alloc);

        alloc.deposit(save_data);
        alloc.shrink_to_fit()
    })
}

pub fn make_bench(
    rng: &mut impl Rng,
    input_size: usize,
) -> impl FnMut() -> usize {
    let input = data::SaveData {
        players: generate_vec(rng, input_size),
    };

    let mut bytes = Align16::frame(10_000_000);
    move || {
        black_box(populate_buffer(
            black_box(&input),
            black_box(bytes.slot().as_bytes()),
        ))
    }
}
