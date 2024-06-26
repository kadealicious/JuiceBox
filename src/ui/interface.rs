use super::{SimTool, UIStateManager, UI_ICON_COUNT};
use bevy::{
    asset::{AssetServer, Handle},
    ecs::{
        event::EventWriter,
        system::{Query, Res, ResMut},
    },
    render::texture::Image,
    window::Window,
};
use bevy_egui::{
    egui::{self, Align2, Color32, Frame, Pos2, Ui, Vec2},
    EguiContexts,
};

use egui::FontFamily::Proportional;
use egui::FontId;
use egui::TextStyle::*;

use crate::{
    events::{ModifyVisualizationEvent, PlayPauseStepEvent},
    file_system::JuiceStates,
};

pub fn init_user_interface(
    mut contexts: EguiContexts,
    asset_server: Res<AssetServer>,
    mut ui_state: ResMut<UIStateManager>,
) {
    load_user_interface_icons(&mut ui_state, &asset_server);
    update_user_interface_style(&mut contexts);
}

/// Modify the user interface style to be a little less inaccessible.
fn update_user_interface_style(contexts: &mut EguiContexts) {
    // Clone the current context and its style to modify it.
    let ctx = contexts.ctx_mut();
    let mut style = (*ctx.style()).clone();

    // Modify styles!
    style.text_styles = [
        (Heading, FontId::new(30.0, Proportional)),
        (Name("Heading2".into()), FontId::new(25.0, Proportional)),
        (Name("Context".into()), FontId::new(23.0, Proportional)),
        (Body, FontId::new(18.0, Proportional)),
        (Monospace, FontId::new(18.0, Proportional)),
        (Button, FontId::new(18.0, Proportional)),
        (Small, FontId::new(14.0, Proportional)),
    ]
    .into();
    style.visuals.override_text_color = Some(Color32::WHITE);
    style.visuals.widgets.inactive.bg_fill = Color32::LIGHT_GRAY;

    // Mutate global style with the above changes.
    ctx.set_style(style);
}

pub fn draw_user_interface(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UIStateManager>,
    windows: Query<&Window>,
    ev_viz: EventWriter<ModifyVisualizationEvent>,
    ev_pause: EventWriter<PlayPauseStepEvent>,
) {
    // Make sure the UI is aware of the window size so we can grow/shrink when needed.
    calculate_window_parameters(&mut ui_state, &mut contexts, windows.single());

    // Show "static" UI menus.
    show_scene_manager_menu(&mut ui_state, &mut contexts);
    show_play_pause_menu(&mut ui_state, &mut contexts, ev_pause);

    // Show hideable UI menus.
    if ui_state.show_selected_tool {
        show_current_tool_menu(&mut ui_state, &mut contexts);
    }
    if ui_state.show_visualization {
        show_visualization_menu(&mut ui_state, &mut contexts, ev_viz);
    }
    if ui_state.show_informational {
        show_informational_menu(&mut ui_state, &mut contexts);
    }
}

/// Create the "splash" menu that appears once when the program is started.
fn show_informational_menu(ui_state: &mut UIStateManager, contexts: &mut EguiContexts) {
    // Create an eGUI window.
    egui::Window::new("Welcome to JuiceBox!")
        .frame(ui_state.window_frame)
        .default_pos(Pos2 {
            x: ui_state.window_size.x / 2.0,
            y: ui_state.window_size.y / 2.0,
        })
        .pivot(Align2::CENTER_CENTER)
        .resizable(false)
        .title_bar(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Welcome to JuiceBox!");
                    ui.end_row();
                    ui.label("(Spilling encouraged)");
                    ui.separator();
                    ui.add_visible(false, egui::Separator::default());
                });

                ui.label("Keyboard controls:");
                ui.end_row();
                ui.label(" • WASD - Move the camera around.");
                ui.end_row();
                ui.label(" • Arrow keys - Play with gravity!");
                ui.end_row();
                ui.label(" • Q & E - Zoom in/out.");
                ui.end_row();
                ui.label(" • R - Reset Simulation.");
                ui.end_row();
                ui.label(" • Space - Pause/unpause.");
                ui.end_row();
                ui.label(" • F (Tap) - Step through the simulation!");
                ui.end_row();

                ui.vertical_centered(|ui| {
                    ui.add_visible(false, egui::Separator::default());
                    ui.separator();

                    if ui.button("Get Spilling!").clicked() {
                        ui_state.show_informational = false;
                    }
                });
            });
        });
}

/// Create menu for file saving/loading and tool selection.
fn show_scene_manager_menu(ui_state: &mut UIStateManager, contexts: &mut EguiContexts) {
    /* For each UI icon that we need to load, get their handle from our UI State Manager.  Then,
    convert that into an eGUI-readable egui::Image format!  This is done by iterating through
    the tool icon handles stores in our UI state manager, and then pushing the eGUI-compatible
    texture handle to our list of tool_icons.  These icons will be iterated over later to draw
    each tool button. */
    /* TODO: Maybe move this out of here so we don't do this every frame?  No idea if that is even
    possible. */
    let mut tool_icons: Vec<egui::Image> = Vec::new();
    for i in 0..UI_ICON_COUNT {
        let icon_handle = ui_state.tool_icon_handles[i].clone_weak();
        tool_icons.push(image_handle_to_egui_texture(
            icon_handle,
            contexts,
            ui_state.icon_size,
        ));
    }

    // Create an eGUI window.
    egui::Window::new("Scene Manager")
        .frame(ui_state.window_frame)
        .fixed_pos(Pos2 { x: 0.0, y: 0.0 })
        .fixed_size(ui_state.window_size)
        .title_bar(false)
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            // Allow the UI windows to grow to the size of the screen.
            ui.set_width(ui_state.window_size.x);
            ui.set_width(ui_state.window_size.y);

            // Show the file manager panel, a horizontal separator, and the tool manager panel.
            show_file_manager_panel(ui_state, ui);
            ui.separator();
            show_tool_manager_panel(ui_state, ui, &tool_icons);
        });
}

/// File management row; align horizontally wrapped.
fn show_file_manager_panel(ui_state: &mut UIStateManager, ui: &mut Ui) {
    ui.horizontal_wrapped(|ui| {
        // "File" scene saving/loading dropdown.
        let file_options = ["File", "New", "Load", "Save", "Save as"];
        let mut file_selection = 0;
        egui::ComboBox::from_id_source(0).show_index(
            ui,
            &mut file_selection,
            file_options.len(),
            |i| file_options[i].to_owned(),
        );
        // Do stuff when selection changes.
        match file_selection {
            1 => ui_state.file_state = JuiceStates::New,
            2 => ui_state.file_state = JuiceStates::Loading,
            3 => ui_state.file_state = JuiceStates::Saving,
            4 => ui_state.file_state = JuiceStates::SavingAs,
            _ => {}
        }

        // "Edit" scene dropdown.
        let edit_options = ["Edit", "Reload", "Clear"];
        let mut edit_selection = 0;
        egui::ComboBox::from_id_source(1).show_index(
            ui,
            &mut edit_selection,
            edit_options.len(),
            |i| edit_options[i].to_owned(),
        );
        // Do stuff when selection changes.
        match edit_selection {
            1 => ui_state.file_state = JuiceStates::Reloading,
            2 => { ui_state.clear = true }
            _ => {}
        }

        // "View" scene dropdown.
        let view_options = ["View", "Tool", "Visuals", "Controls"];
        let mut view_selection = 0;
        egui::ComboBox::from_id_source(2).show_index(
            ui,
            &mut view_selection,
            view_options.len(),
            |i| view_options[i].to_owned(),
        );
        // Do stuff when selection changes.
        match view_selection {
            1 => ui_state.show_selected_tool = !ui_state.show_selected_tool,
            2 => ui_state.show_visualization = !ui_state.show_visualization,
            3 => ui_state.show_informational = !ui_state.show_informational,
            _ => {}
        }

        if ui.button("Help!").clicked() {
            ui_state.show_informational = !ui_state.show_informational;
        }
    });
}

/// Scene/tool management row; align horizontally wrapped.
fn show_tool_manager_panel(
    ui_state: &mut UIStateManager,
    ui: &mut Ui,
    tool_icons: &Vec<egui::Image>,
) {
    ui.horizontal_wrapped(|ui| {
        // Draw each tool button from our list!
        for i in 0..UI_ICON_COUNT {
            let current_tool: SimTool = i.into();

            /* If the currently selected tool corresponds to the button we are creating, highlight
            and add it to the UI.  Otherwise, only add it to the UI. */
            if ui_state.selected_tool == current_tool {
                // Store the previous text color and switch it to black for readability.
                let prev_text_color = ui.visuals().text_color();
                ui.visuals_mut().override_text_color = Some(Color32::BLACK);

                // Create a highlighted button with the new text color.
                let button =
                    egui::Button::image_and_text(tool_icons[i].clone(), current_tool.as_str())
                        .fill(Color32::GOLD);

                // Add the button to our UI!
                ui.add(button);

                // Reset text color.
                ui.visuals_mut().override_text_color = Some(prev_text_color);
            } else {
                // Otherwise if the button is not clicked, draw it unhighlighted.
                let button =
                    egui::Button::image_and_text(tool_icons[i].clone(), current_tool.as_str());

                // Add our button to the UI and switch the active tool when it is clicked!
                if ui.add(button).clicked() {
                    ui_state.selected_tool = current_tool;
                }
            }
        }
    });
}

/// Show the menu with the current tool's options.
fn show_current_tool_menu(ui_state: &mut UIStateManager, contexts: &mut EguiContexts) {
    // Get the currently selected tool's name.
    let selected_tool_name: String = ui_state.selected_tool.as_str().to_owned();
    let context_window_name: String = selected_tool_name + " Options";

    // Create a new eGUI window.
    egui::Window::new(context_window_name)
        .id(egui::Id::from("Tool Selection Window"))
        .frame(ui_state.window_frame)
        .pivot(Align2::CENTER_CENTER)
        .default_pos(Pos2 {
            x: 0.0,
            y: ui_state.window_size.y / 2.0,
        })
        .default_width(0.0)
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            // Align the buttons in this row horizontally from left to right.
            ui.with_layout(egui::Layout::top_down(egui::Align::BOTTOM), |ui| {
                // Show different buttons depending on which tool is currently selected.
                match ui_state.selected_tool {
                    // For the Move Camera tool, show a slider for the grabbing radius.
                    SimTool::Camera => {
                        ui.label("Click and drag (or use WASD) to move the camera around!");
                    }

                    // For the Zoom tool, show a slider for the zooming radius.
                    SimTool::Zoom => {
                        ui.label("Use the Q and E keys to zoom in and out!");

                        ui.add(
                            egui::Slider::new(&mut ui_state.zoom_slider, 0.5..=5.0).text("Zoom!"),
                        );
                    }

                    // For the Gravity tool, show sliders for the gravity strength and direction.
                    SimTool::Gravity => {
                        ui.label(
                            "Use the arrow keys to rotate and change the strength of gravity!",
                        );

                        ui.add(
                            egui::Slider::new(&mut ui_state.gravity_direction, 0.0..=360.0)
                                .text("Gravity Direction"),
                        );

                        ui.add(
                            egui::Slider::new(&mut ui_state.gravity_magnitude, 0.0001..=20.0)
                                .text("Gravity Strength"),
                        );
                    }

                    // For the Grab tool, show a slider for the grabbing radius.
                    SimTool::Grab => {
                        ui.add(
                            egui::Slider::new(&mut ui_state.grab_slider_radius, 5.0..=100.0)
                                .text("Grab Radius"),
                        );
                    }

                    // For the Add Fluid tool, show density and radius sliders.
                    SimTool::AddFluid => {
                        ui.add(
                            egui::Slider::new(&mut ui_state.add_remove_fluid_radius, 1.0..=50.0)
                                .text("Brush Radius"),
                        );
                        ui.add(
                            egui::Slider::new(&mut ui_state.add_fluid_density, 0.01..=1.0)
                                .text("Fluid Density"),
                        );
                    }

                    // For the Remove Fluid tool, show a radius slider.
                    SimTool::RemoveFluid => {
                        ui.add(
                            egui::Slider::new(&mut ui_state.add_remove_fluid_radius, 1.0..=50.0)
                                .text("Eraser Radius"),
                        );
                    }

                    // For the Add Wall tool, show some text as there are no options for Add Wall.
                    SimTool::AddWall => {
                        ui.label("Click anywhere in the simulation to add a wall!");
                    }

                    // For the Remove Wall tool, show some text as there are no options for Remove Wall.
                    SimTool::RemoveWall => {
                        ui.label("Click a wall in the simulation to remove it!");
                    }

                    /* For the Add Faucet tool, show sliders for the direction, volume, and speed
                    of the fluid coming out of the faucet. */
                    SimTool::AddFaucet => {
                        ui.add(
                            egui::Slider::new(&mut ui_state.faucet_direction, 0.0..=360.0)
                                .text("Faucet Direction"),
                        );
                        // This does not appear to do the simulation any favors!  Bye bye!
                        // ui.add(egui::Slider::new(
                        // 	&mut ui_state.faucet_radius,
                        // 	0.01..=2.5
                        // ).text("Faucet Pipe Diameter"));
                        ui.add(
                            egui::Slider::new(&mut ui_state.faucet_pressure, 0.0..=100.0)
                                .text("Faucet Pressure"),
                        );
                    }

                    // For the Remove Faucet tool, show some text as there are no options for Remove Faucet.
                    SimTool::RemoveFaucet => {
                        ui.label("Click a faucet in the simulation to remove it!");
                    }

                    /* For the Add Drain tool, show a sucking radius radius slider and a pressure slider
                    for controlling how intensely a drain pulls fluid inwards. */
                    SimTool::AddDrain => {
                        ui.add(
                            egui::Slider::new(&mut ui_state.drain_radius, 0.0..=35.0)
                                .text("Drain Suck Radius"),
                        );
                        ui.add(
                            egui::Slider::new(&mut ui_state.drain_pressure, 0.0..=50.0)
                                .text("Drain Pressure"),
                        );
                    }

                    // For the Remove Drain tool, show some text as there are no options for Remove Drain.
                    SimTool::RemoveDrain => {
                        ui.label("Click a drain in the simulation to remove it!");
                    } // // It should literally not be possible for this final case to happen.
                      // _ => {
                      //     ui.label("If you are seeing this message, something is wrong :(");
                      // }
                }
            });
        });
}

/// Grid/fluid visualization settings menu.
fn show_visualization_menu(
    ui_state: &mut UIStateManager,
    contexts: &mut EguiContexts,
    mut ev_viz: EventWriter<ModifyVisualizationEvent>,
) {
    // Whenever our visualization is modified, update this variable and send an event out.
    let mut viz_mod: bool = false;

    egui::Window::new("Visualization Options")
        .frame(ui_state.window_frame)
        .pivot(Align2::CENTER_CENTER)
        .default_pos(Pos2 {
            x: ui_state.window_size.x,
            y: ui_state.window_size.y / 2.0,
        })
        .default_width(0.0)
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            // Align the buttons in this row horizontally from left to right.
            ui.with_layout(egui::Layout::top_down(egui::Align::TOP), |ui| {
                if ui.checkbox(&mut ui_state.show_grid, "Show Grid").clicked() {
                    viz_mod = true;
                }
                if ui
                    .checkbox(&mut ui_state.show_velocity_vectors, "Show Velocities")
                    .clicked()
                {
                    viz_mod = true;
                }
                if ui
                    .checkbox(&mut ui_state.show_gravity_vector, "Show Gravity")
                    .clicked()
                {
                    viz_mod = true;
                }

                ui.separator();

                // Fluid color visualization option dropdown.
                ui.horizontal_wrapped(|ui| {
                    // Labels for each button.
                    ui.label("Color by:");
                    let color_options = ["Velocity", "Density", "None"];

                    // Combobox setup and event polling:
                    if egui::ComboBox::from_id_source(0)
                        .show_index(
                            ui,
                            &mut ui_state.fluid_color_variable,
                            color_options.len(),
                            |i| color_options[i].to_owned(),
                        )
                        .changed()
                    {
                        viz_mod = true;
                    }
                });

                // Fluid color pickers.
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .color_edit_button_rgb(&mut ui_state.fluid_colors[0])
                        .changed()
                    {
                        viz_mod = true;
                    }
                    if ui
                        .color_edit_button_rgb(&mut ui_state.fluid_colors[1])
                        .changed()
                    {
                        viz_mod = true;
                    }
                    if ui
                        .color_edit_button_rgb(&mut ui_state.fluid_colors[2])
                        .changed()
                    {
                        viz_mod = true;
                    }
                    if ui
                        .color_edit_button_rgb(&mut ui_state.fluid_colors[3])
                        .changed()
                    {
                        viz_mod = true;
                    }
                });

                ui.separator();

                // Sliders for the particle size and gravity direction.
                if ui
                    .add(
                        egui::Slider::new(&mut ui_state.particle_physical_size, 0.2..=2.0)
                            .text("Particle Size"),
                    )
                    .changed()
                {
                    viz_mod = true;
                }
            });
        });

    if viz_mod {
        ev_viz.send(ModifyVisualizationEvent::new(ui_state));
    }
}

/// Play/pause menu.
fn show_play_pause_menu(
    ui_state: &mut UIStateManager,
    contexts: &mut EguiContexts,
    mut ev_pause: EventWriter<PlayPauseStepEvent>,
) {
    // Get the icons we need!
    let play_icon = image_handle_to_egui_texture(
        ui_state.play_pause_icon_handles[0].clone_weak(),
        contexts,
        ui_state.icon_size,
    );
    let pause_icon = image_handle_to_egui_texture(
        ui_state.play_pause_icon_handles[1].clone_weak(),
        contexts,
        ui_state.icon_size,
    );

    egui::Window::new("Play/Pause")
        .title_bar(false)
        .frame(ui_state.window_frame)
        .fixed_pos(Pos2 {
            x: ui_state.window_size.x / 2.0,
            y: ui_state.window_size.y * 0.95,
        })
        .pivot(Align2::CENTER_CENTER)
        .default_width(105.0)
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            // Simulation play/pause button.
            ui.vertical_centered(|ui| {
                // Play/pause button icon and text.
                let play_pause_icon;
                let play_pause_text;
                if ui_state.is_paused {
                    play_pause_icon = play_icon;
                    play_pause_text = "Paused!";
                } else {
                    play_pause_icon = pause_icon;
                    play_pause_text = "Playing!";
                }

                // The actual button itself.
                if ui
                    .add(egui::Button::image_and_text(
                        play_pause_icon,
                        play_pause_text,
                    ))
                    .clicked()
                {
                    ui_state.is_paused = !ui_state.is_paused;
                    ev_pause.send(PlayPauseStepEvent::new(false));
                }
            });
        });
}

/// Determine the size and frame of the drawing window and store it in our UI state manager.
fn calculate_window_parameters(
    ui_state: &mut UIStateManager,
    contexts: &mut EguiContexts,
    window: &Window,
) {
    // General styling of components for consistency.
    let window_border_width: f32 = 2.5;
    let window_padding: f32 = 10.0;

    // Figure out how large our window is that we are drawing to.
    ui_state.window_size = Vec2 {
        x: window.width() - window_padding - window_border_width,
        y: window.height(),
    };
    ui_state.window_frame = Frame {
        fill: contexts.ctx_mut().style().visuals.window_fill(),
        rounding: 10.0.into(),
        stroke: contexts
            .ctx_mut()
            .style()
            .visuals
            .widgets
            .noninteractive
            .fg_stroke,
        inner_margin: (window_padding / 2.0).into(),
        outer_margin: 0.5.into(), // so the stroke is within the bounds
        ..Default::default()
    };
}

/// Using Bevy's asset server, load all UI icons into our UI state manager.
pub fn load_user_interface_icons(ui_state: &mut UIStateManager, asset_server: &AssetServer) {
    // Load all UI icons using Bevy's asset server.
    let icon_handles: [Handle<Image>; UI_ICON_COUNT] = [
        asset_server.load("../assets/ui/movecamera.png"),
        asset_server.load("../assets/ui/zoom.png"),
        asset_server.load("../assets/ui/rotate.png"),
        asset_server.load("../assets/ui/grab.png"),
        asset_server.load("../assets/ui/addfluid.png"),
        asset_server.load("../assets/ui/removefluid.png"),
        asset_server.load("../assets/ui/addwall.png"),
        asset_server.load("../assets/ui/removewall.png"),
        asset_server.load("../assets/ui/addfaucet.png"),
        asset_server.load("../assets/ui/removefaucet.png"),
        asset_server.load("../assets/ui/adddrain.png"),
        asset_server.load("../assets/ui/removedrain.png"),
    ];
    let play_pause_icon_handles: [Handle<Image>; 2] = [
        asset_server.load("../assets/ui/play.png"),
        asset_server.load("../assets/ui/pause.png"),
    ];

    // Store all loaded image handles into our UI state manager.
    for i in 0..UI_ICON_COUNT {
        ui_state.tool_icon_handles[i] = icon_handles[i].clone();
    }
    ui_state.play_pause_icon_handles[0] = play_pause_icon_handles[0].clone();
    ui_state.play_pause_icon_handles[1] = play_pause_icon_handles[1].clone();
}

/// Convert a Bevy Handle<Image> into an eGUI-compatible eGUI Image!
fn image_handle_to_egui_texture<'a>(
    image_handle: Handle<Image>,
    contexts: &mut EguiContexts,
    size: Vec2,
) -> bevy_egui::egui::Image<'a> {
    // Add the image to our eGUI context from our UI state manager.
    let select_icon_id = contexts.add_image(image_handle);

    // Convert the eGUI texture ID into an image that eGUI can actually draw.
    let select_icon_img =
        egui::widgets::Image::new(egui::load::SizedTexture::new(select_icon_id, size));

    select_icon_img
}
