use super::{
    hotbar,
    img_ids::{Imgs, ImgsRot},
    item_imgs::ItemImgs,
    slots, BarNumbers, ShortcutNumbers, BLACK, CRITICAL_HP_COLOR, HP_COLOR, LOW_HP_COLOR,
    STAMINA_COLOR, TEXT_COLOR, UI_HIGHLIGHT_0,
};
use crate::{
    i18n::Localization,
    ui::{
        fonts::Fonts,
        slot::{ContentSize, SlotMaker},
        ImageFrame, Tooltip, TooltipManager, Tooltipable,
    },
    window::GameInput,
    GlobalState,
};
use common::comp::{
    inventory::slot::EquipSlot,
    item::{
        tool::{AbilityMap, Tool, ToolKind},
        Hands, ItemKind,
    },
    Energy, Health, Inventory,
};
use conrod_core::{
    color,
    widget::{self, Button, Image, Rectangle, Text},
    widget_ids, Color, Colorable, Positionable, Sizeable, Widget, WidgetCommon,
};
use vek::*;

widget_ids! {
    struct Ids {
        // Death message
        death_message_1,
        death_message_2,
        death_message_1_bg,
        death_message_2_bg,
        death_bg,
        // Level up message
        level_up,
        level_down,
        level_align,
        level_message,
        level_message_bg,
        // Hurt BG
        hurt_bg,
        // Skillbar
        alignment,
        bg,
        frame,
        bg_health,
        frame_health,
        bg_stamina,
        frame_stamina,
        m1_ico,
        m2_ico,
        // Level
        level_bg,
        level,
        // Exp-Bar
        exp_alignment,
        exp_filling,
        // HP-Bar
        hp_alignment,
        hp_filling,
        hp_txt_alignment,
        hp_txt_bg,
        hp_txt,
        // Stamina-Bar
        stamina_alignment,
        stamina_filling,
        stamina_txt_alignment,
        stamina_txt_bg,
        stamina_txt,
        // Slots
        m1_slot,
        m1_slot_bg,
        m1_text,
        m1_text_bg,
        m1_slot_act,
        m1_content,
        m2_slot,
        m2_slot_bg,
        m2_text,
        m2_text_bg,
        m2_slot_act,
        m2_content,
        slot1,
        slot1_text,
        slot1_text_bg,
        slot2,
        slot2_text,
        slot2_text_bg,
        slot3,
        slot3_text,
        slot3_text_bg,
        slot4,
        slot4_text,
        slot4_text_bg,
        slot5,
        slot5_text,
        slot5_text_bg,
        slot6,
        slot6_text,
        slot6_text_bg,
        slot7,
        slot7_text,
        slot7_text_bg,
        slot8,
        slot8_text,
        slot8_text_bg,
        slot9,
        slot9_text,
        slot9_text_bg,
        slot10,
        slot10_text,
        slot10_text_bg,
    }
}

#[derive(WidgetCommon)]
pub struct Skillbar<'a> {
    global_state: &'a GlobalState,
    imgs: &'a Imgs,
    item_imgs: &'a ItemImgs,
    fonts: &'a Fonts,
    rot_imgs: &'a ImgsRot,
    health: &'a Health,
    inventory: &'a Inventory,
    energy: &'a Energy,
    // character_state: &'a CharacterState,
    // controller: &'a ControllerInputs,
    hotbar: &'a hotbar::State,
    tooltip_manager: &'a mut TooltipManager,
    slot_manager: &'a mut slots::SlotManager,
    localized_strings: &'a Localization,
    pulse: f32,
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    ability_map: &'a AbilityMap,
}

impl<'a> Skillbar<'a> {
    #[allow(clippy::too_many_arguments)] // TODO: Pending review in #587
    pub fn new(
        global_state: &'a GlobalState,
        imgs: &'a Imgs,
        item_imgs: &'a ItemImgs,
        fonts: &'a Fonts,
        rot_imgs: &'a ImgsRot,
        health: &'a Health,
        inventory: &'a Inventory,
        energy: &'a Energy,
        // character_state: &'a CharacterState,
        pulse: f32,
        // controller: &'a ControllerInputs,
        hotbar: &'a hotbar::State,
        tooltip_manager: &'a mut TooltipManager,
        slot_manager: &'a mut slots::SlotManager,
        localized_strings: &'a Localization,
        ability_map: &'a AbilityMap,
    ) -> Self {
        Self {
            global_state,
            imgs,
            item_imgs,
            fonts,
            rot_imgs,
            health,
            inventory,
            energy,
            common: widget::CommonBuilder::default(),
            // character_state,
            pulse,
            // controller,
            hotbar,
            tooltip_manager,
            slot_manager,
            localized_strings,
            ability_map,
        }
    }
}

pub struct State {
    ids: Ids,
}

impl<'a> Widget for Skillbar<'a> {
    type Event = ();
    type State = State;
    type Style = ();

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
        }
    }

    #[allow(clippy::unused_unit)] // TODO: Pending review in #587
    fn style(&self) -> Self::Style { () }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { state, ui, .. } = args;

        let mut hp_percentage = self.health.current() as f64 / self.health.maximum() as f64 * 100.0;
        let mut energy_percentage =
            self.energy.current() as f64 / self.energy.maximum() as f64 * 100.0;
        if self.health.is_dead {
            hp_percentage = 0.0;
            energy_percentage = 0.0;
        };

        let bar_values = self.global_state.settings.gameplay.bar_numbers;
        let shortcuts = self.global_state.settings.gameplay.shortcut_numbers;

        let hp_ani = (self.pulse * 4.0/* speed factor */).cos() * 0.5 + 0.8; //Animation timer
        let crit_hp_color: Color = Color::Rgba(0.79, 0.19, 0.17, hp_ani);

        let localized_strings = self.localized_strings;

        let slot_offset = 3.0;

        // Death message
        if self.health.is_dead {
            if let Some(key) = self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Respawn)
            {
                Text::new(&localized_strings.get("hud.you_died"))
                    .middle_of(ui.window)
                    .font_size(self.fonts.cyri.scale(50))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
                    .set(state.ids.death_message_1_bg, ui);
                Text::new(
                    &localized_strings
                        .get("hud.press_key_to_respawn")
                        .replace("{key}", key.to_string().as_str()),
                )
                .mid_bottom_with_margin_on(state.ids.death_message_1_bg, -120.0)
                .font_size(self.fonts.cyri.scale(30))
                .font_id(self.fonts.cyri.conrod_id)
                .color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
                .set(state.ids.death_message_2_bg, ui);
                Text::new(&localized_strings.get("hud.you_died"))
                    .bottom_left_with_margins_on(state.ids.death_message_1_bg, 2.0, 2.0)
                    .font_size(self.fonts.cyri.scale(50))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(CRITICAL_HP_COLOR)
                    .set(state.ids.death_message_1, ui);
                Text::new(
                    &localized_strings
                        .get("hud.press_key_to_respawn")
                        .replace("{key}", key.to_string().as_str()),
                )
                .bottom_left_with_margins_on(state.ids.death_message_2_bg, 2.0, 2.0)
                .font_size(self.fonts.cyri.scale(30))
                .font_id(self.fonts.cyri.conrod_id)
                .color(CRITICAL_HP_COLOR)
                .set(state.ids.death_message_2, ui);
            }
        }
        // Skillbar
        // Alignment and BG
        let alignment_size = 40.0 * 12.0 + slot_offset * 11.0;
        Rectangle::fill_with([alignment_size, 80.0], color::TRANSPARENT)
            .mid_bottom_with_margin_on(ui.window, 10.0)
            .set(state.ids.frame, ui);
        // Health and Stamina bar
        let show_health = self.health.current() != self.health.maximum();
        let show_stamina = self.energy.current() != self.energy.maximum();

        if show_health && !self.health.is_dead {
            let offset = 1.0;
            Image::new(self.imgs.health_bg)
                .w_h(484.0, 24.0)
                .mid_top_with_margin_on(state.ids.frame, -offset)
                .set(state.ids.bg_health, ui);
            Rectangle::fill_with([480.0, 18.0], color::TRANSPARENT)
                .top_left_with_margins_on(state.ids.bg_health, 2.0, 2.0)
                .set(state.ids.hp_alignment, ui);
            let health_col = match hp_percentage as u8 {
                0..=20 => crit_hp_color,
                21..=40 => LOW_HP_COLOR,
                _ => HP_COLOR,
            };
            Image::new(self.imgs.bar_content)
                .w_h(480.0 * hp_percentage / 100.0, 18.0)
                .color(Some(health_col))
                .top_left_with_margins_on(state.ids.hp_alignment, 0.0, 0.0)
                .set(state.ids.hp_filling, ui);
            Image::new(self.imgs.health_frame)
                .w_h(484.0, 24.0)
                .color(Some(UI_HIGHLIGHT_0))
                .middle_of(state.ids.bg_health)
                .set(state.ids.frame_health, ui);
        }
        if show_stamina && !self.health.is_dead {
            let offset = if show_health { 34.0 } else { 1.0 };
            Image::new(self.imgs.stamina_bg)
                .w_h(323.0, 16.0)
                .mid_top_with_margin_on(state.ids.frame, -offset)
                .set(state.ids.bg_stamina, ui);
            Rectangle::fill_with([319.0, 10.0], color::TRANSPARENT)
                .top_left_with_margins_on(state.ids.bg_stamina, 2.0, 2.0)
                .set(state.ids.stamina_alignment, ui);
            Image::new(self.imgs.bar_content)
                .w_h(319.0 * energy_percentage / 100.0, 10.0)
                .color(Some(STAMINA_COLOR))
                .top_left_with_margins_on(state.ids.stamina_alignment, 0.0, 0.0)
                .set(state.ids.stamina_filling, ui);
            Image::new(self.imgs.stamina_frame)
                .w_h(323.0, 16.0)
                .color(Some(UI_HIGHLIGHT_0))
                .middle_of(state.ids.bg_stamina)
                .set(state.ids.frame_stamina, ui);
        }
        // Bar Text
        // Values
        if let BarNumbers::Values = bar_values {
            let mut hp_txt = format!(
                "{}/{}",
                (self.health.current() / 10).max(1) as u32, /* Don't show 0 health for
                                                             * living players */
                (self.health.maximum() / 10) as u32
            );
            let mut energy_txt = format!(
                "{}/{}",
                (self.energy.current() / 10) as u32,
                (self.energy.maximum() / 10) as u32
            );
            if self.health.is_dead {
                hp_txt = self.localized_strings.get("hud.group.dead").to_string();
                energy_txt = self.localized_strings.get("hud.group.dead").to_string();
            };
            Text::new(&hp_txt)
                .middle_of(state.ids.frame_health)
                .font_size(self.fonts.cyri.scale(12))
                .font_id(self.fonts.cyri.conrod_id)
                .color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
                .set(state.ids.hp_txt_bg, ui);
            Text::new(&hp_txt)
                .bottom_left_with_margins_on(state.ids.hp_txt_bg, 2.0, 2.0)
                .font_size(self.fonts.cyri.scale(12))
                .font_id(self.fonts.cyri.conrod_id)
                .color(TEXT_COLOR)
                .set(state.ids.hp_txt, ui);
            Text::new(&energy_txt)
                .middle_of(state.ids.frame_stamina)
                .font_size(self.fonts.cyri.scale(12))
                .font_id(self.fonts.cyri.conrod_id)
                .color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
                .set(state.ids.stamina_txt_bg, ui);
            Text::new(&energy_txt)
                .bottom_left_with_margins_on(state.ids.stamina_txt_bg, 2.0, 2.0)
                .font_size(self.fonts.cyri.scale(12))
                .font_id(self.fonts.cyri.conrod_id)
                .color(TEXT_COLOR)
                .set(state.ids.stamina_txt, ui);
        }
        //Percentages
        if let BarNumbers::Percent = bar_values {
            let mut hp_txt = format!("{}%", hp_percentage as u32);
            let mut energy_txt = format!("{}", energy_percentage as u32);
            if self.health.is_dead {
                hp_txt = self.localized_strings.get("hud.group.dead").to_string();
                energy_txt = self.localized_strings.get("hud.group.dead").to_string();
            };
            Text::new(&hp_txt)
                .middle_of(state.ids.frame_health)
                .font_size(self.fonts.cyri.scale(12))
                .font_id(self.fonts.cyri.conrod_id)
                .color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
                .set(state.ids.hp_txt_bg, ui);
            Text::new(&hp_txt)
                .bottom_left_with_margins_on(state.ids.hp_txt_bg, 2.0, 2.0)
                .font_size(self.fonts.cyri.scale(12))
                .font_id(self.fonts.cyri.conrod_id)
                .color(TEXT_COLOR)
                .set(state.ids.hp_txt, ui);
            Text::new(&energy_txt)
                .middle_of(state.ids.frame_stamina)
                .font_size(self.fonts.cyri.scale(12))
                .font_id(self.fonts.cyri.conrod_id)
                .color(Color::Rgba(0.0, 0.0, 0.0, 1.0))
                .set(state.ids.stamina_txt_bg, ui);
            Text::new(&energy_txt)
                .bottom_left_with_margins_on(state.ids.stamina_txt_bg, 2.0, 2.0)
                .font_size(self.fonts.cyri.scale(12))
                .font_id(self.fonts.cyri.conrod_id)
                .color(TEXT_COLOR)
                .set(state.ids.stamina_txt, ui);
        }
        // Slots
        let content_source = (self.hotbar, self.inventory, self.energy, self.ability_map); // TODO: avoid this
        let image_source = (self.item_imgs, self.imgs);
        let mut slot_maker = SlotMaker {
            // TODO: is a separate image needed for the frame?
            empty_slot: self.imgs.skillbar_slot,
            filled_slot: self.imgs.skillbar_slot,
            selected_slot: self.imgs.inv_slot_sel,
            background_color: None,
            content_size: ContentSize {
                width_height_ratio: 1.0,
                max_fraction: 0.8, /* Changes the item image size by setting a maximum fraction
                                    * of either the width or height */
            },
            selected_content_scale: 1.0,
            amount_font: self.fonts.cyri.conrod_id,
            amount_margins: Vec2::new(1.0, 1.0),
            amount_font_size: self.fonts.cyri.scale(12),
            amount_text_color: TEXT_COLOR,
            content_source: &content_source,
            image_source: &image_source,
            slot_manager: Some(self.slot_manager),
            pulse: self.pulse,
        };
        let item_tooltip = Tooltip::new({
            // Edge images [t, b, r, l]
            // Corner images [tr, tl, br, bl]
            let edge = &self.rot_imgs.tt_side;
            let corner = &self.rot_imgs.tt_corner;
            ImageFrame::new(
                [edge.cw180, edge.none, edge.cw270, edge.cw90],
                [corner.none, corner.cw270, corner.cw90, corner.cw180],
                Color::Rgba(0.08, 0.07, 0.04, 1.0),
                5.0,
            )
        })
        .title_font_size(self.fonts.cyri.scale(15))
        .parent(ui.window)
        .desc_font_size(self.fonts.cyri.scale(12))
        .font_id(self.fonts.cyri.conrod_id)
        .desc_text_color(TEXT_COLOR);
        // Helper
        let tooltip_text = |slot| {
            content_source
                .0
                .get(slot)
                .and_then(|content| match content {
                    hotbar::SlotContents::Inventory(i) => content_source
                        .1
                        .get(i)
                        .map(|item| (item.name(), item.description())),
                    hotbar::SlotContents::Ability3 => content_source
                        .1
                        .equipped(EquipSlot::Mainhand)
                        .map(|i| i.kind())
                        .and_then(|kind| match kind {
                            ItemKind::Tool(Tool { kind, .. }) => match kind {
                                ToolKind::Hammer => Some((
                                    "Smash of Doom",
                                    "\nAn AOE attack with knockback. \nLeaps to position of \
                                     cursor.",
                                )),
                                ToolKind::Axe => {
                                    Some(("Spin Leap", "\nA slashing running spin leap."))
                                },
                                ToolKind::Staff => Some((
                                    "Firebomb",
                                    "\nWhirls a big fireball into the air. \nExplodes the ground \
                                     and does\na big amount of damage",
                                )),
                                ToolKind::Sword => Some((
                                    "Whirlwind",
                                    "\nMove forward while spinning with \n your sword.",
                                )),
                                ToolKind::Bow => Some((
                                    "Burst",
                                    "\nLaunches a burst of arrows at the top \nof a running leap.",
                                )),
                                ToolKind::Debug => Some((
                                    "Possessing Arrow",
                                    "\nShoots a poisonous arrow.\nLets you control your target.",
                                )),
                                _ => None,
                            },
                            _ => None,
                        }),
                })
        };
        // Slot 1-5
        // Slot 1
        slot_maker.empty_slot = self.imgs.skillbar_slot;
        slot_maker.selected_slot = self.imgs.skillbar_slot;
        let slot = slot_maker
            .fabricate(hotbar::Slot::One, [40.0; 2])
            .filled_slot(self.imgs.skillbar_slot)
            .bottom_left_with_margins_on(state.ids.frame, 0.0, 0.0);
        if let Some((title, desc)) = tooltip_text(hotbar::Slot::One) {
            slot.with_tooltip(self.tooltip_manager, title, desc, &item_tooltip, TEXT_COLOR)
                .set(state.ids.slot1, ui);
        } else {
            slot.set(state.ids.slot1, ui);
        }
        // Slot 2
        let slot = slot_maker
            .fabricate(hotbar::Slot::Two, [40.0; 2])
            .filled_slot(self.imgs.skillbar_slot)
            .right_from(state.ids.slot1, slot_offset);
        if let Some((title, desc)) = tooltip_text(hotbar::Slot::Two) {
            slot.with_tooltip(self.tooltip_manager, title, desc, &item_tooltip, TEXT_COLOR)
                .set(state.ids.slot2, ui);
        } else {
            slot.set(state.ids.slot2, ui);
        }
        // Slot 3
        let slot = slot_maker
            .fabricate(hotbar::Slot::Three, [40.0; 2])
            .filled_slot(self.imgs.skillbar_slot)
            .right_from(state.ids.slot2, slot_offset);
        if let Some((title, desc)) = tooltip_text(hotbar::Slot::Three) {
            slot.with_tooltip(self.tooltip_manager, title, desc, &item_tooltip, TEXT_COLOR)
                .set(state.ids.slot3, ui);
        } else {
            slot.set(state.ids.slot3, ui);
        }
        // Slot 4
        let slot = slot_maker
            .fabricate(hotbar::Slot::Four, [40.0; 2])
            .filled_slot(self.imgs.skillbar_slot)
            .right_from(state.ids.slot3, slot_offset);
        if let Some((title, desc)) = tooltip_text(hotbar::Slot::Four) {
            slot.with_tooltip(self.tooltip_manager, title, desc, &item_tooltip, TEXT_COLOR)
                .set(state.ids.slot4, ui);
        } else {
            slot.set(state.ids.slot4, ui);
        }
        // Slot 5
        let slot = slot_maker
            .fabricate(hotbar::Slot::Five, [40.0; 2])
            .filled_slot(self.imgs.skillbar_slot)
            .right_from(state.ids.slot4, slot_offset);
        if let Some((title, desc)) = tooltip_text(hotbar::Slot::Five) {
            slot.with_tooltip(self.tooltip_manager, title, desc, &item_tooltip, TEXT_COLOR)
                .set(state.ids.slot5, ui);
        } else {
            slot.set(state.ids.slot5, ui);
        }
        // Slot M1
        Image::new(self.imgs.skillbar_slot)
            .w_h(40.0, 40.0)
            .right_from(state.ids.slot5, slot_offset)
            .set(state.ids.m1_slot_bg, ui);
        Button::image(
            match self
                .inventory
                .equipped(EquipSlot::Mainhand)
                .map(|i| i.kind())
            {
                Some(ItemKind::Tool(Tool { kind, .. })) => match kind {
                    ToolKind::Sword => self.imgs.twohsword_m1,
                    ToolKind::Dagger => self.imgs.onehdagger_m1,
                    ToolKind::Shield => self.imgs.onehshield_m1,
                    ToolKind::Hammer => self.imgs.twohhammer_m1,
                    ToolKind::Axe => self.imgs.twohaxe_m1,
                    ToolKind::Bow => self.imgs.bow_m1,
                    ToolKind::Sceptre => self.imgs.heal_0,
                    ToolKind::Staff => self.imgs.fireball,
                    ToolKind::Debug => self.imgs.flyingrod_m1,
                    _ => self.imgs.nothing,
                },
                _ => self.imgs.nothing,
            },
        ) // Insert Icon here
        .w_h(36.0, 36.0)
        .middle_of(state.ids.m1_slot_bg)
        .set(state.ids.m1_content, ui);
        // Slot M2
        Image::new(self.imgs.skillbar_slot)
            .w_h(40.0, 40.0)
            .right_from(state.ids.m1_slot_bg, slot_offset)
            .set(state.ids.m2_slot_bg, ui);

        fn get_tool(inventory: &Inventory, equip_slot: EquipSlot) -> Option<&Tool> {
            match inventory.equipped(equip_slot).map(|i| i.kind()) {
                Some(ItemKind::Tool(tool)) => Some(tool),
                _ => None,
            }
        }

        let active_tool = get_tool(self.inventory, EquipSlot::Mainhand);
        let second_tool = get_tool(self.inventory, EquipSlot::Offhand);

        let tool = match (active_tool.map(|x| x.hands), second_tool.map(|x| x.hands)) {
            (Some(Hands::TwoHand), _) => active_tool,
            (_, Some(Hands::OneHand)) => second_tool,
            (Some(Hands::OneHand), _) => active_tool,
            (_, _) => None,
        };

        Button::image(match tool.map(|t| t.kind) {
            Some(ToolKind::Sword) => self.imgs.twohsword_m2,
            Some(ToolKind::Dagger) => self.imgs.onehdagger_m2,
            Some(ToolKind::Shield) => self.imgs.onehshield_m2,
            Some(ToolKind::Hammer) => self.imgs.hammergolf,
            Some(ToolKind::Axe) => self.imgs.axespin,
            Some(ToolKind::Bow) => self.imgs.bow_m2,
            Some(ToolKind::Sceptre) => self.imgs.heal_bomb,
            Some(ToolKind::Staff) => self.imgs.flamethrower,
            Some(ToolKind::Debug) => self.imgs.flyingrod_m2,
            _ => self.imgs.nothing,
        })
        .w_h(36.0, 36.0)
        .middle_of(state.ids.m2_slot_bg)
        .image_color(if let Some(tool) = tool {
            if self.energy.current()
                >= tool
                    .get_abilities(self.ability_map)
                    .secondary
                    .get_energy_cost()
            {
                Color::Rgba(1.0, 1.0, 1.0, 1.0)
            } else {
                Color::Rgba(0.3, 0.3, 0.3, 0.8)
            }
        } else {
            match tool.map(|t| t.kind) {
                None => Color::Rgba(1.0, 1.0, 1.0, 0.0),
                _ => Color::Rgba(1.0, 1.0, 1.0, 1.0),
            }
        })
        .set(state.ids.m2_content, ui);
        // Slot 6-10
        // Slot 6
        slot_maker.empty_slot = self.imgs.skillbar_slot;
        slot_maker.selected_slot = self.imgs.skillbar_slot;
        let slot = slot_maker
            .fabricate(hotbar::Slot::Six, [40.0; 2])
            .filled_slot(self.imgs.skillbar_slot)
            .right_from(state.ids.m2_slot_bg, slot_offset);
        if let Some((title, desc)) = tooltip_text(hotbar::Slot::Six) {
            slot.with_tooltip(self.tooltip_manager, title, desc, &item_tooltip, TEXT_COLOR)
                .set(state.ids.slot6, ui);
        } else {
            slot.set(state.ids.slot6, ui);
        }
        // Slot 7
        let slot = slot_maker
            .fabricate(hotbar::Slot::Seven, [40.0; 2])
            .filled_slot(self.imgs.skillbar_slot)
            .right_from(state.ids.slot6, slot_offset);
        if let Some((title, desc)) = tooltip_text(hotbar::Slot::Seven) {
            slot.with_tooltip(self.tooltip_manager, title, desc, &item_tooltip, TEXT_COLOR)
                .set(state.ids.slot7, ui);
        } else {
            slot.set(state.ids.slot7, ui);
        }
        // Slot 8
        let slot = slot_maker
            .fabricate(hotbar::Slot::Eight, [40.0; 2])
            .filled_slot(self.imgs.skillbar_slot)
            .right_from(state.ids.slot7, slot_offset);
        if let Some((title, desc)) = tooltip_text(hotbar::Slot::Eight) {
            slot.with_tooltip(self.tooltip_manager, title, desc, &item_tooltip, TEXT_COLOR)
                .set(state.ids.slot8, ui);
        } else {
            slot.set(state.ids.slot8, ui);
        }
        // Slot 9
        let slot = slot_maker
            .fabricate(hotbar::Slot::Nine, [40.0; 2])
            .filled_slot(self.imgs.skillbar_slot)
            .right_from(state.ids.slot8, slot_offset);
        if let Some((title, desc)) = tooltip_text(hotbar::Slot::Nine) {
            slot.with_tooltip(self.tooltip_manager, title, desc, &item_tooltip, TEXT_COLOR)
                .set(state.ids.slot9, ui);
        } else {
            slot.set(state.ids.slot9, ui);
        }
        // Quickslot
        slot_maker.empty_slot = self.imgs.skillbar_slot;
        slot_maker.selected_slot = self.imgs.skillbar_slot;
        let slot = slot_maker
            .fabricate(hotbar::Slot::Ten, [40.0; 2])
            .filled_slot(self.imgs.skillbar_slot)
            .right_from(state.ids.slot9, slot_offset);
        if let Some((title, desc)) = tooltip_text(hotbar::Slot::Ten) {
            slot.with_tooltip(self.tooltip_manager, title, desc, &item_tooltip, TEXT_COLOR)
                .set(state.ids.slot10, ui);
        } else {
            slot.set(state.ids.slot10, ui);
        }
        // Shortcuts
        if let ShortcutNumbers::On = shortcuts {
            if let Some(slot1) = &self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Slot1)
            {
                Text::new(slot1.to_string().as_str())
                    .top_right_with_margins_on(state.ids.slot1, 3.0, 5.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(BLACK)
                    .set(state.ids.slot1_text_bg, ui);
                Text::new(slot1.to_string().as_str())
                    .bottom_left_with_margins_on(state.ids.slot1_text_bg, 1.0, 1.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(TEXT_COLOR)
                    .set(state.ids.slot1_text, ui);
            }
            if let Some(slot2) = &self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Slot2)
            {
                Text::new(slot2.to_string().as_str())
                    .top_right_with_margins_on(state.ids.slot2, 3.0, 5.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(BLACK)
                    .set(state.ids.slot2_text_bg, ui);
                Text::new(slot2.to_string().as_str())
                    .bottom_left_with_margins_on(state.ids.slot2_text_bg, 1.0, 1.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(TEXT_COLOR)
                    .set(state.ids.slot2_text, ui);
            }
            if let Some(slot3) = &self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Slot3)
            {
                Text::new(slot3.to_string().as_str())
                    .top_right_with_margins_on(state.ids.slot3, 3.0, 5.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(BLACK)
                    .set(state.ids.slot3_text_bg, ui);
                Text::new(slot3.to_string().as_str())
                    .bottom_left_with_margins_on(state.ids.slot3_text_bg, 1.0, 1.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(TEXT_COLOR)
                    .set(state.ids.slot3_text, ui);
            }
            if let Some(slot4) = &self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Slot4)
            {
                Text::new(slot4.to_string().as_str())
                    .top_right_with_margins_on(state.ids.slot4, 3.0, 5.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(BLACK)
                    .set(state.ids.slot4_text_bg, ui);
                Text::new(slot4.to_string().as_str())
                    .bottom_left_with_margins_on(state.ids.slot4_text_bg, 1.0, 1.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(TEXT_COLOR)
                    .set(state.ids.slot4_text, ui);
            }
            if let Some(slot5) = &self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Slot5)
            {
                Text::new(slot5.to_string().as_str())
                    .top_right_with_margins_on(state.ids.slot5, 3.0, 5.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(BLACK)
                    .set(state.ids.slot5_text_bg, ui);
                Text::new(slot5.to_string().as_str())
                    .bottom_left_with_margins_on(state.ids.slot5_text_bg, 1.0, 1.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(TEXT_COLOR)
                    .set(state.ids.slot5_text, ui);
            }
            if let Some(slot6) = &self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Slot6)
            {
                Text::new(slot6.to_string().as_str())
                    .top_right_with_margins_on(state.ids.slot6, 3.0, 5.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(BLACK)
                    .set(state.ids.slot6_text_bg, ui);
                Text::new(slot6.to_string().as_str())
                    .bottom_right_with_margins_on(state.ids.slot6_text_bg, 1.0, 1.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(TEXT_COLOR)
                    .set(state.ids.slot6_text, ui);
            }
            if let Some(slot7) = &self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Slot7)
            {
                Text::new(slot7.to_string().as_str())
                    .top_right_with_margins_on(state.ids.slot7, 3.0, 5.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(BLACK)
                    .set(state.ids.slot7_text_bg, ui);
                Text::new(slot7.to_string().as_str())
                    .bottom_right_with_margins_on(state.ids.slot7_text_bg, 1.0, 1.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(TEXT_COLOR)
                    .set(state.ids.slot7_text, ui);
            }
            if let Some(slot8) = &self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Slot8)
            {
                Text::new(slot8.to_string().as_str())
                    .top_right_with_margins_on(state.ids.slot8, 3.0, 5.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(BLACK)
                    .set(state.ids.slot8_text_bg, ui);
                Text::new(slot8.to_string().as_str())
                    .bottom_right_with_margins_on(state.ids.slot8_text_bg, 1.0, 1.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(TEXT_COLOR)
                    .set(state.ids.slot8_text, ui);
            }
            if let Some(slot9) = &self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Slot9)
            {
                Text::new(slot9.to_string().as_str())
                    .top_right_with_margins_on(state.ids.slot9, 3.0, 5.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(BLACK)
                    .set(state.ids.slot9_text_bg, ui);
                Text::new(slot9.to_string().as_str())
                    .bottom_right_with_margins_on(state.ids.slot9_text_bg, 1.0, 1.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(TEXT_COLOR)
                    .set(state.ids.slot9_text, ui);
            }
            if let Some(slot10) = &self
                .global_state
                .settings
                .controls
                .get_binding(GameInput::Slot10)
            {
                Text::new(slot10.to_string().as_str())
                    .top_right_with_margins_on(state.ids.slot10, 3.0, 5.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(BLACK)
                    .set(state.ids.slot10_text_bg, ui);
                Text::new(slot10.to_string().as_str())
                    .bottom_right_with_margins_on(state.ids.slot10_text_bg, 1.0, 1.0)
                    .font_size(self.fonts.cyri.scale(8))
                    .font_id(self.fonts.cyri.conrod_id)
                    .color(TEXT_COLOR)
                    .set(state.ids.slot10_text, ui);
            }
        };
        // M1 and M2 icons
        Image::new(self.imgs.m1_ico)
            .w_h(16.0, 18.0)
            .mid_bottom_with_margin_on(state.ids.m1_content, -11.0)
            .set(state.ids.m1_ico, ui);
        Image::new(self.imgs.m2_ico)
            .w_h(16.0, 18.0)
            .mid_bottom_with_margin_on(state.ids.m2_content, -11.0)
            .set(state.ids.m2_ico, ui);
    }
}
