use crate::{contracts::*, input::CQRSInput};
use axum::{body::HttpBody, handler::Handler, routing::post, Router};
use serde::Serialize;
use std::future::Future;

pub trait CommandHandler<M, T> {}
pub trait QueryHandler<M, T> {}

pub trait CQRSBuilder<S, B> {
    fn command<H, T, TC, R>(self, handler: H) -> Self
    where
        TC: Command,
        H: Handler<T, S, B> + CommandHandler<R, TC>,
        T: 'static;

    fn query<H, T, Q, R>(self, handler: H) -> Self
    where
        Q: Query,
        H: Handler<T, S, B> + QueryHandler<R, Q>,
        T: 'static;
}

impl<S, B> CQRSBuilder<S, B> for Router<S, B>
where
    S: Clone + Send + Sync + 'static,
    B: HttpBody + Send + 'static,
{
    fn command<H, T, C, R>(self, handler: H) -> Self
    where
        C: Command,
        H: Handler<T, S, B> + CommandHandler<R, C>,
        T: 'static,
    {
        self.route(&format!("/command/{}", C::name()), post(handler))
    }

    fn query<H, T, Q, R>(self, handler: H) -> Self
    where
        Q: Query,
        H: Handler<T, S, B> + QueryHandler<R, Q>,
        T: 'static,
    {
        self.route(&format!("/query/{}", Q::name()), post(handler))
    }
}

macro_rules! impl_handlers {
    (
        [$($ty:ident),*], $last:ident
    ) => {
        impl<F, Fut, $($ty,)* $last> CommandHandler<($($ty,)* $last,), $last> for F
        where
            F: FnOnce($($ty,)* CQRSInput<$last>,) -> Fut,
            Fut: Future<Output = CommandResult<$last>> ,
            $last: Command + Serialize,
        {}

        impl<F, Fut, $($ty,)* $last> QueryHandler<($($ty,)* $last,), $last> for F
        where
            F: FnOnce($($ty,)* CQRSInput<$last>,) -> Fut,
            Fut: Future<Output = QueryResult<$last>> ,
            $last: Query + Serialize,
        {}
    };
}

impl_handlers!([], T1);
impl_handlers!([T1], T2);
impl_handlers!([T1, T2], T3);
impl_handlers!([T1, T2, T3], T4);
impl_handlers!([T1, T2, T3, T4], T5);
impl_handlers!([T1, T2, T3, T4, T5], T6);
