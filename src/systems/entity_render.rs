use crate::prelude::*;

// render an entity that has a Point and a Render component 
// takes a writable lease on Point, read only on Render
#[system]
#[write_component(Point)]
#[read_component(Render)]
pub fn entity_render(ecs: &SubWorld, #[resource] camera: &Camera) {
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(1);
    let offset = Point::new(camera.left_x, camera.top_y);

    // match any entity with Point and Render
    <(&Point, &Render)>::query()
        .iter(ecs)
        .for_each(|(pos, render)| {
            draw_batch.set(*pos - offset, render.colour, render.glyph);
        });

    draw_batch.submit(5000).expect("Batch error")
}
