//! Composable layouts.

mod abs;
mod align;
mod angle;
mod axes;
mod columns;
mod container;
mod corners;
mod dir;
mod em;
mod flow;
mod fr;
mod fragment;
mod frame;
mod grid;
mod hide;
mod inline;
#[path = "layout.rs"]
mod layout_;
mod length;
#[path = "measure.rs"]
mod measure_;
mod pad;
mod page;
mod place;
mod point;
mod ratio;
mod regions;
mod rel;
mod repeat;
mod sides;
mod size;
mod spacing;
mod stack;
mod transform;

pub use self::abs::*;
pub use self::align::*;
pub use self::angle::*;
pub use self::axes::*;
pub use self::columns::*;
pub use self::container::*;
pub use self::corners::*;
pub use self::dir::*;
pub use self::em::*;
pub use self::flow::*;
pub use self::fr::*;
pub use self::fragment::*;
pub use self::frame::*;
pub use self::grid::*;
pub use self::hide::*;
pub use self::layout_::*;
pub use self::length::*;
pub use self::measure_::*;
pub use self::pad::*;
pub use self::page::*;
pub use self::place::*;
pub use self::point::*;
pub use self::ratio::*;
pub use self::regions::*;
pub use self::rel::*;
pub use self::repeat::*;
pub use self::sides::*;
pub use self::size::*;
pub use self::spacing::*;
pub use self::stack::*;
pub use self::transform::*;

pub(crate) use self::inline::*;

use comemo::{Tracked, TrackedMut};

use crate::diag::{bail, SourceResult};
use crate::engine::{Engine, Route};
use crate::eval::Tracer;
use crate::foundations::{category, Category, Content, Scope, StyleChain};
use crate::introspection::{Introspector, Locator};
use crate::model::Document;
use crate::realize::{realize_doc, realize_flow, Arenas};
use crate::World;

/// Arranging elements on the page in different ways.
///
/// By combining layout functions, you can create complex and automatic layouts.
#[category]
pub static LAYOUT: Category;

/// Hook up all `layout` definitions.
pub fn define(global: &mut Scope) {
    global.category(LAYOUT);
    global.define_type::<Length>();
    global.define_type::<Angle>();
    global.define_type::<Ratio>();
    global.define_type::<Rel<Length>>();
    global.define_type::<Fr>();
    global.define_type::<Dir>();
    global.define_type::<Alignment>();
    global.define_elem::<PageElem>();
    global.define_elem::<PagebreakElem>();
    global.define_elem::<VElem>();
    global.define_elem::<HElem>();
    global.define_elem::<BoxElem>();
    global.define_elem::<BlockElem>();
    global.define_elem::<StackElem>();
    global.define_elem::<GridElem>();
    global.define_elem::<ColumnsElem>();
    global.define_elem::<ColbreakElem>();
    global.define_elem::<PlaceElem>();
    global.define_elem::<FlushElem>();
    global.define_elem::<AlignElem>();
    global.define_elem::<PadElem>();
    global.define_elem::<RepeatElem>();
    global.define_elem::<MoveElem>();
    global.define_elem::<ScaleElem>();
    global.define_elem::<RotateElem>();
    global.define_elem::<HideElem>();
    global.define_func::<measure>();
    global.define_func::<layout>();
}

impl Content {
    /// Layout the content into a document.
    ///
    /// This first realizes the content into a
    /// [`DocumentElem`][crate::model::DocumentElem], which is then laid out. In
    /// contrast to [`layout`](Self::layout()), this does not take regions since
    /// the regions are defined by the page configuration in the content and
    /// style chain.
    pub fn layout_document(
        &self,
        engine: &mut Engine,
        styles: StyleChain,
    ) -> SourceResult<Document> {
        #[comemo::memoize]
        fn cached(
            content: &Content,
            world: Tracked<dyn World + '_>,
            introspector: Tracked<Introspector>,
            route: Tracked<Route>,
            locator: Tracked<Locator>,
            tracer: TrackedMut<Tracer>,
            styles: StyleChain,
        ) -> SourceResult<Document> {
            let mut locator = Locator::chained(locator);
            let mut engine = Engine {
                world,
                introspector,
                route: Route::extend(route).unnested(),
                locator: &mut locator,
                tracer,
            };
            let arenas = Arenas::default();
            let (document, styles) = realize_doc(&mut engine, &arenas, content, styles)?;
            document.layout(&mut engine, styles)
        }

        cached(
            self,
            engine.world,
            engine.introspector,
            engine.route.track(),
            engine.locator.track(),
            TrackedMut::reborrow_mut(&mut engine.tracer),
            styles,
        )
    }

    /// Layout the content into the given regions.
    pub fn layout(
        &self,
        engine: &mut Engine,
        styles: StyleChain,
        regions: Regions,
    ) -> SourceResult<Fragment> {
        let fragment = self.measure(engine, styles, regions)?;
        engine.locator.visit_frames(&fragment);
        Ok(fragment)
    }

    /// Layout without side effects.
    ///
    /// For the results to be valid, the element must either be layouted again
    /// or the measurement must be confirmed through a call to
    /// `engine.locator.visit_frames(&fragment)`.
    pub fn measure(
        &self,
        engine: &mut Engine,
        styles: StyleChain,
        regions: Regions,
    ) -> SourceResult<Fragment> {
        #[allow(clippy::too_many_arguments)]
        #[comemo::memoize]
        fn cached(
            content: &Content,
            world: Tracked<dyn World + '_>,
            introspector: Tracked<Introspector>,
            route: Tracked<Route>,
            locator: Tracked<Locator>,
            tracer: TrackedMut<Tracer>,
            styles: StyleChain,
            regions: Regions,
        ) -> SourceResult<Fragment> {
            let mut locator = Locator::chained(locator);
            let mut engine = Engine {
                world,
                introspector,
                route: Route::extend(route),
                locator: &mut locator,
                tracer,
            };

            if !engine.route.within(Route::MAX_LAYOUT_DEPTH) {
                bail!(
                    content.span(), "maximum layout depth exceeded";
                    hint: "try to reduce the amount of nesting in your layout",
                );
            }

            // If we are in a `PageElem`, this might already be a realized flow.
            if let Some(flow) = content.to_packed::<FlowElem>() {
                return flow.layout(&mut engine, styles, regions);
            }

            // Layout the content by first turning it into a `FlowElem` and then
            // layouting that.
            let arenas = Arenas::default();
            let (flow, styles) = realize_flow(&mut engine, &arenas, content, styles)?;
            flow.layout(&mut engine, styles, regions)
        }

        cached(
            self,
            engine.world,
            engine.introspector,
            engine.route.track(),
            engine.locator.track(),
            TrackedMut::reborrow_mut(&mut engine.tracer),
            styles,
            regions,
        )
    }
}
