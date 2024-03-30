use std::{borrow::Cow, sync::Arc};
use super::{RouteSection, RouteSections};
use crate::{
    fang::{BoxedFPC, Fangs},
    handler::{ByAnother, Handler, Handlers},
};


#[derive(Clone, Debug)]
pub struct TrieRouter {
    pub(super) id:   RouterID,
    pub(super) root: Node,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct RouterID(usize);
impl RouterID {
    fn new() -> Self {
        use std::sync::{OnceLock, Mutex};

        static ID: OnceLock<Mutex<usize>> = OnceLock::new();

        let mut id_lock = ID
            .get_or_init(|| Mutex::new(0))
            .lock().unwrap();

        let id = *id_lock + 1;
        *id_lock = id;
        drop(id_lock);

        Self(id)
    }
}

#[derive(Clone/* for testing */)]
pub(super) struct Node {
    /// Why Option: root node doesn't have pattern
    pub(super) pattern:    Option<Pattern>,
    pub(super) handlers:   HandlerMap,
    pub(super) fangs_list: FangsList,
    pub(super) children:   Vec<Node>,
} const _: () = {
    impl std::fmt::Debug for Node {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("")
                .field("pattern",    &self.pattern)
                .field("handler",    &self.handlers)
                .field("fangs_list", &self.fangs_list.iter().map(|_| '#').collect::<Vec<_>>())
                .field("children",   &self.children)
                .finish()
        }
    }
};

#[derive(Clone, Debug)]
pub(super) struct HandlerMap {
    GET:     Option<Handler>,
    PUT:     Option<Handler>,
    POST:    Option<Handler>,
    PATCH:   Option<Handler>,
    DELETE:  Option<Handler>,
    OPTIONS: Option<Handler>,
}

#[derive(Clone)]
pub(super) enum Pattern {
    Static(Cow<'static, [u8]>),
    Param,
} const _: () = {
    impl std::fmt::Debug for Pattern {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Param     => f.write_str(":Param"),
                Self::Static(v) => f.write_str(&format!(
                    "'{}'", std::str::from_utf8(&v).unwrap()
                )),
            }
        }
    }

    impl PartialEq for Pattern {
        fn eq(&self, other: &Self) -> bool {
            match self {
                Self::Param => match other {
                    Self::Param => true,
                    _ => false,
                }
                Self::Static(this_bytes) => {
                    match other {
                        Self::Static(other_bytes) => this_bytes == other_bytes,
                        _ => false
                    }
                }
            }
        }
    }

    impl From<RouteSection> for Pattern {
        fn from(section: RouteSection) -> Self {
            match section {
                RouteSection::Param         => Pattern::Param,
                RouteSection::Static(bytes) => Pattern::Static(Cow::Borrowed(bytes))
            }
        }
    }
};

#[derive(Clone)]
pub(super) struct FangsList(Vec<(
    RouterID,
    Arc<dyn Fangs>)
>);
impl FangsList {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn add(&mut self, id: RouterID, fangs: Arc<dyn Fangs>) {
        if self.0.iter().find(|(_id, _)| *_id == id).is_none() {
            self.0.push((id, fangs));
        }
    }
    fn extend(&mut self, another: Self) {
        for (id, fangs) in another.0.into_iter() {
            self.add(id, fangs)
        }
    }

    fn into_proc_with(self, handler: Handler) -> BoxedFPC {
        let mut iter = self.into_iter();

        match iter.next() {
            None => BoxedFPC::from_proc(handler),
            Some(most_inner) => iter.fold(
                most_inner.build_handler(handler),
                |proc, fangs| fangs.build(proc)
            )
        }
    }

    /// yield from most inner fangs
    fn iter(&self) -> impl Iterator<Item = &Arc<dyn Fangs>> {
        self.0.iter()
            .map(|(_, fangs)| fangs)
    }
    /// yield from most inner fangs
    fn into_iter(self) -> impl Iterator<Item = Arc<dyn Fangs>> {
        self.0.into_iter()
            .map(|(_, fangs)| fangs)
    }
}
const _: () = {
    impl std::fmt::Debug for FangsList {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut d = f.debug_tuple("FangsList");
            let mut d = &mut d;
            for (id, _) in self.0.iter() {
                d = d.field(id);
            }
            d.finish()
        }
    }
};


impl TrieRouter {
    pub(crate) fn new() -> Self {
        Self {
            id:   RouterID::new(),
            root: Node::root(),
        }
    }

    pub(crate) fn id(&self) -> RouterID {
        self.id.clone()
    }

    pub(crate) fn register_handlers(&mut self, handlers: Handlers) {
        let Handlers { route, GET, PUT, POST, PATCH, DELETE } = handlers;

        if let Some(handler) = GET {
            if let Err(e) = self.GET.register_handler(route.clone().into_iter(), handler) {panic!("{e}")}
        }
        if let Some(handler) = PUT {
            if let Err(e) = self.PUT.register_handler(route.clone().into_iter(), handler) {panic!("{e}")}
        }
        if let Some(handler) = POST {
            if let Err(e) = self.POST.register_handler(route.clone().into_iter(), handler) {panic!("{e}")}
        }
        if let Some(handler) = PATCH {
            if let Err(e) = self.PATCH.register_handler(route.clone().into_iter(), handler) {panic!("{e}")}
        }
        if let Some(handler) = DELETE {
            if let Err(e) = self.DELETE.register_handler(route.clone().into_iter(), handler) {panic!("{e}")}
        }
    }

    pub(crate) fn apply_fangs(&mut self, id: RouterID, fangs: Arc<dyn Fangs>) {
        self.GET   .apply_fangs(id.clone(), fangs.clone());
        self.PUT   .apply_fangs(id.clone(), fangs.clone());
        self.POST  .apply_fangs(id.clone(), fangs.clone());
        self.PATCH .apply_fangs(id.clone(), fangs.clone());
        self.DELETE.apply_fangs(id.clone(), fangs.clone());

        self.OPTIONS.add(id, fangs);
    }

    pub(crate) fn merge_another(&mut self, another: ByAnother) {
        let ByAnother { route, ohkami } = another;
        let another_routes = ohkami.into_router();

        self.GET   .merge_node(route.clone().into_iter(), another_routes.GET   ).unwrap();
        self.PUT   .merge_node(route.clone().into_iter(), another_routes.PUT   ).unwrap();
        self.POST  .merge_node(route.clone().into_iter(), another_routes.POST  ).unwrap();
        self.PATCH .merge_node(route.clone().into_iter(), another_routes.PATCH ).unwrap();
        self.DELETE.merge_node(route.clone().into_iter(), another_routes.DELETE).unwrap();
    }

    pub(crate) fn into_radix(self) -> super::RadixRouter {
        super::RadixRouter {
            GET:     self.GET    .into_radix(),
            PUT:     self.PUT    .into_radix(),
            POST:    self.POST   .into_radix(),
            PATCH:   self.PATCH  .into_radix(),
            DELETE:  self.DELETE .into_radix(),
            OPTIONS: self.OPTIONS.into_proc_with(Handler::default_no_content()),
        }
    }
}

impl Node {
    fn register_handler(&mut self, mut route: <RouteSections as IntoIterator>::IntoIter, handler: Handler) -> Result<(), String> {
        match route.next() {
            None => {
                self.set_handler(handler)?;
                Ok(())
            }
            Some(pattern)   => match self.machable_child_mut(pattern.clone().into()) {
                Some(child) => child.register_handler(route, handler),
                None        => {
                    let mut child = Node::new(pattern.into());
                    child.register_handler(route, handler)?;
                    self.append_child(child)?;
                    Ok(())
                }
            }
        }
    }

    fn merge_node(
        &mut self,
        mut route_to_merge_root: <RouteSections as IntoIterator>::IntoIter,
        another: Node,
    ) -> Result<(), String> {
        match route_to_merge_root.next() {
            None => {
                self.merge_here(another)?;
                Ok(())
            }
            Some(pattern) => match self.machable_child_mut(pattern.clone().into()) {
                Some(child) => child.merge_node(route_to_merge_root, another),
                None => {
                    let mut new_child = Node::new(pattern.into());
                    new_child.merge_node(route_to_merge_root, another)?;
                    self.append_child(new_child)?;
                    Ok(())
                }
            }
        }
    }

    /// MUST be called after all handlers are registered
    fn apply_fangs(&mut self, id: RouterID, fangs: Arc<dyn Fangs>) {
        for child in &mut self.children {
            child.apply_fangs(id.clone(), fangs.clone())
        }

        self.fangs_list.add(id, fangs);
    }

    fn into_radix(self) -> super::radix::Node {
        let Node { pattern, mut fangs_list, mut handler, mut children } = self;

        let mut patterns = pattern.into_iter().collect::<Vec<_>>();

        while children.len() == 1 && handler.is_none() {
            let Node {
                pattern:    child_pattern,
                fangs_list: child_fangses,
                handler:    child_handler,
                children:   child_children,
            } = children.pop(/* pop the single child */).unwrap(/* `children` is empty here */);

            children = child_children;

            handler  = child_handler;

            fangs_list.extend(child_fangses);
            
            let child_pattern = child_pattern.unwrap(/* `child` is not root */);
            if patterns.last().is_some_and(|last| last.is_static()) && child_pattern.is_static() {
                let last_pattern = patterns.pop(/*=== POPing here ===*/).unwrap();
                let this_static  = last_pattern.to_static().unwrap();
                let child_static = child_pattern.to_static().unwrap();

                patterns.push(Pattern::Static(
                    Cow::Owned([this_static, b"/", child_static].concat())
                ));
            } else {
                patterns.push(child_pattern)
            }
        }

        children.sort_unstable_by(|a, b| match (a.pattern.as_ref().unwrap(), b.pattern.as_ref().unwrap()) {
            (Pattern::Static(_), Pattern::Param) => std::cmp::Ordering::Less,
            (Pattern::Param, Pattern::Static(_)) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal
        });

        let catch_proc  = fangs_list.clone().into_proc_with(Handler::default_not_found());
        let handle_proc = fangs_list.into_proc_with(handler.unwrap_or(Handler::default_not_found()));

        super::radix::Node {
            catch_proc,
            handle_proc,
            patterns: Box::leak(patterns.into_iter().map(Pattern::into_radix).collect()),
            children: children.into_iter().map(Node::into_radix).collect::<Box<[_]>>(),
        }
    }
}


/*===== utils =====*/


impl Node {
    fn new(pattern: Pattern) -> Self {
        Self {
            pattern:    Some(pattern),
            handler:    None,
            fangs_list: FangsList::new(),
            children:   vec![],
        }
    }
    fn root() -> Self {
        Self {
            pattern:    None,
            handler:    None,
            fangs_list: FangsList::new(),
            children:   vec![],
        }
    }

    fn machable_child_mut(&mut self, pattern: Pattern) -> Option<&mut Node> {
        for child in &mut self.children {
            if child.pattern.as_ref().unwrap().matches(&pattern) {
                return Some(child)
            }
        }
        None
    }

    /// Called in following situation :
    /// 
    /// <br/>
    /// 
    /// ```ignore
    /// TrieRouter::new()
    ///     .register_handlers("/hc".GET(health_check))
    ///     .register_handlers("/api".by(
    ///         Ohkami::new((
    ///             "/"         .GET (hello),
    ///             "/users"    .POST(create_user),
    ///             "/users/:id".GET (get_user),
    ///             "/tasks"    .GET (get_task),
    ///         ))
    ///     ))
    /// ```
    /// 
    /// <br/>
    /// 
    /// This must equals to :
    /// 
    /// <br/>
    /// 
    /// ```ignore
    /// TrieRouter::new()
    ///     .register_handlers("/hc"           .GET (health_check))
    ///     .register_handlers("/api"          .GET (hello))
    ///     .register_handlers("/api/users"    .POST(create_user))
    ///     .register_handlers("/api/users/:id".GET (get_user))
    ///     .register_handlers("/api/tasks/:id".GET (get_task));
    /// ```
    fn merge_here(&mut self, another_root: Node) -> Result<(), String> {
        if self.handler.is_some() {
            return Err(format!(
                "Can't merge another Ohkami at route that already has handler"
            ))
        }

        let Node {
            pattern:  None, // <-- another_root must be a root node and has pattern `None`
            fangs_list:  another_root_fangses,
            handler:  another_root_handler,
            children: another_children,
        } = another_root else {
            panic!("Unexpectedly called `Node::merge_here` where `another_root` is not root node")
        };

        if let Some(new_handler) = another_root_handler {
            self.set_handler(new_handler)?
        }

        self.append_fangs(another_root_fangses);

        for ac in another_children {
            self.append_child(ac)?
        }

        Ok(())
    }
}

impl Node {
    fn append_child(&mut self, new_child: Node) -> Result<(), String> {
        match new_child.pattern.as_ref().expect("Invalid child node: Child node must have pattern") {
            Pattern::Param => {
                self.children.push(new_child);
                Ok(())
            }
            Pattern::Static(bytes) => {
                if self.children.iter().find(|c| c.pattern.as_ref().unwrap().to_static().is_some_and(|p| p == bytes.as_ref())).is_some() {
                    let __position__ = match &self.pattern {
                        None    => format!("For the first part of route"),
                        Some(p) => format!("After {p:?}"),
                    };
                    Err(format!("Conflicting route definition: {__position__}, pattern '{}' is registered twice", std::str::from_utf8(&bytes).unwrap()))
                } else {
                    self.children.push(new_child);
                    Ok(())
                }
            }
        }
    }

    fn append_fangs(&mut self, fangs: FangsList) {
        self.fangs_list.extend(fangs);
    }

    fn set_handler(&mut self, new_handler: Handler) -> Result<(), String> {
        if self.handler.is_some() {
            return Err(format!("Conflicting handler registering"))
        }
        self.handler = Some(new_handler);
        Ok(())
    }
}

impl Pattern {
    fn is_param(&self) -> bool {
        match self {
            Self::Param => true,
            Self::Static(_) => false,
        }
    }
    fn is_static(&self) -> bool {
        match self {
            Self::Static(_) => true,
            Self::Param => false,
        }
    }

    fn to_static(&self) -> Option<&[u8]> {
        match self {
            Self::Param         => None,
            Self::Static(bytes) => Some(&bytes)
        }
    }

    fn matches(&self, another: &Self) -> bool {
        match self {
            Self::Param => another.is_param(),
            Self::Static{..} => self.to_static() == another.to_static(),
        }
    }

    fn into_radix(self) -> super::radix::Pattern {
        match self {
            Self::Param                        => super::radix::Pattern::Param,
            Self::Static(Cow::Borrowed(bytes)) => super::radix::Pattern::Static(bytes),
            Self::Static(Cow::Owned(vec))      => super::radix::Pattern::Static(vec.leak()),
        }
    }
}
