use egui::{Frame, Response, Sense, Widget};
use extend::ext;

#[ext]
pub impl Frame {
    fn show_and<R>(
        &self,
        ui: &mut egui::Ui,
        add_contents: impl FnOnce(&egui::Ui) -> R,
        accept_response: impl FnOnce(&mut Frame, &mut Response),
    ) {
        let mut prepared = self.begin(ui);
        add_contents(&prepared.content_ui);
        let mut response = prepared.allocate_space(ui);
        accept_response(&mut prepared.frame, &mut response);
        prepared.paint(ui);
    }
}

// let mut make_painter_frame =
//     |do_paint: &dyn Fn(&Painter, Pos2), on_click: &dyn Fn()| {
//         Frame::new()
//             .inner_margin(size)
//             .fill(fill)
//             .corner_radius(2.0)
//             .show_and(
//                 ui,
//                 |ui| {
//                     let pos = ui.next_widget_position();
//                     do_paint(&ui.painter(), pos);
//                 },
//                 |frame, response| {
//                     response.sense.set(Sense::CLICK, true);
//                     if response.hovered() {
//                         frame.stroke = hover_stroke;
//                     }
//                     if response.clicked_by(PointerButton::Primary) {
//                         on_click()
//                     }
//                 },
//             )
//     };
//
// make_painter_frame(
//     &|painter, pos| {
//         painter.arrow(
//             pos + [0.0, size / 2.0].into(),
//             [0.0, -size].into(),
//             paint_stroke,
//         );
//     },
//     &|| info!("Up"),
// );

pub struct PainterFrame<'click, 'paint> {
    pub desired_size: Option<egui::Vec2>,
    pub on_click_handler: Option<Box<dyn FnMut() + 'click>>,
    pub paint_handler: Box<dyn FnMut(&egui::Painter, &egui::Rect, f32) + 'paint>,
}

impl<'click, 'paint> PainterFrame<'click, 'paint> {
    pub fn new(paint_handler: impl FnMut(&egui::Painter, &egui::Rect, f32) + 'paint) -> Self {
        PainterFrame {
            desired_size: None,
            on_click_handler: None,
            paint_handler: Box::new(paint_handler),
        }
    }

    pub fn desired_size(mut self, size: egui::Vec2) -> Self {
        self.desired_size = Some(size);
        self
    }
    pub fn on_click<'a>(mut self, handler: impl FnMut() + 'click) -> Self {
        self.on_click_handler = Some(Box::new(handler));
        self
    }
}

impl<'click, 'paint> Widget for PainterFrame<'click, 'paint> {
    fn ui(mut self, ui: &mut egui::Ui) -> Response {
        let desired_size = self
            .desired_size
            .unwrap_or(ui.spacing().interact_size.y * egui::vec2(1.0, 1.0));
        let (rect, response) =
            ui.allocate_exact_size(desired_size, Sense::click() | Sense::hover());
        if let Some(mut on_click_handler) = self.on_click_handler
            && response.clicked()
        {
            on_click_handler();
        }
        // response.widget_info(|| {
        //     egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *on, "")
        // });

        if ui.is_rect_visible(rect) {
            let hover_progress = ui
                .ctx()
                .animate_bool_responsive(response.id, response.hovered());
            let visuals = ui
                .style()
                .interact_selectable(&response, response.hovered());
            let rect = rect.expand(visuals.expansion);
            ui.painter().rect(
                rect,
                visuals.corner_radius,
                visuals.bg_fill,
                visuals.bg_stroke,
                egui::StrokeKind::Inside,
            );

            (self.paint_handler)(&ui.painter(), &rect, hover_progress);
        }

        response
    }
}
