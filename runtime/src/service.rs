pub enum ServiceError<RoutingError, PollError> {
    Routing(RoutingError),
    Poll(PollError),
}
