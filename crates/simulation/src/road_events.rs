//! Atomic runtime closures: preserve district connectivity and every in-flight unit.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Closure {
    pub incident: IncidentId,
    pub until: u32,
}
impl Simulation {
    pub fn close_road(
        &mut self,
        incident: IncidentId,
        road: u32,
        seconds: u32,
    ) -> Result<(), Error> {
        if !(1..=600).contains(&seconds) || !self.incident(incident)?.escalated {
            return Err(Error::InvalidTransition);
        }
        let original = self.network.as_ref().ok_or(Error::NoRoute)?;
        let edge = original
            .roads
            .iter()
            .find(|r| r.id == road)
            .ok_or(Error::NoRoute)?;
        if edge.closed {
            return Err(Error::InvalidTransition);
        }
        let endpoints = [
            original.node_position(edge.from_node).unwrap(),
            original.node_position(edge.to_node).unwrap(),
        ];
        for u in self.units.values() {
            let elapsed = match u.state {
                UnitState::EnRoute { depart_at, .. } | UnitState::Returning { depart_at, .. } => {
                    self.now.saturating_sub(depart_at)
                }
                _ => 0,
            };
            if u.journey
                .as_ref()
                .is_some_and(|j| j.road_at(elapsed) == Some(road))
                && !endpoints.contains(&u.position)
            {
                return Err(Error::NoRoute);
            }
        }
        let mut network = original.clone();
        network
            .roads
            .iter_mut()
            .find(|r| r.id == road)
            .unwrap()
            .closed = true;
        if !network.round_trip_connected(self.units[&1].station) {
            return Err(Error::NoRoute);
        }
        let mut plans = vec![];
        for u in self.units.values() {
            let (to, elapsed, incident) = match u.state {
                UnitState::EnRoute {
                    to,
                    depart_at,
                    incident,
                    ..
                } => (to, self.now.saturating_sub(depart_at), Some(incident)),
                UnitState::Returning { depart_at, .. } => {
                    (u.station, self.now.saturating_sub(depart_at), None)
                }
                _ => continue,
            };
            let source = u.journey.as_ref().and_then(|j| j.road_at(elapsed));
            let journey = network.route(u.position, to, u.speed_milli, source)?;
            plans.push((u.id, to, incident, journey));
        }
        self.network = Some(network);
        self.closures.insert(
            road,
            Closure {
                incident,
                until: self.now + seconds,
            },
        );
        self.emit(EventKind::RoadClosed {
            incident,
            road,
            reopen_at: self.now + seconds,
        });
        for (id, to, incident, journey) in plans {
            let from = self.units[&id].position;
            let arrive_at = self.now + journey.seconds();
            self.units.get_mut(&id).unwrap().journey = Some(journey);
            let state = if let Some(incident) = incident {
                UnitState::EnRoute {
                    incident,
                    from,
                    to,
                    depart_at: self.now,
                    arrive_at,
                }
            } else {
                UnitState::Returning {
                    from,
                    depart_at: self.now,
                    arrive_at,
                }
            };
            self.change_unit(id, state);
        }
        Ok(())
    }
    pub(super) fn close_near_blast(&mut self, incident: IncidentId) {
        let Some(network) = &self.network else {
            return;
        };
        let location = self.incidents[&incident].location;
        let mut candidates: Vec<_> = network
            .roads
            .iter()
            .filter(|r| !r.closed)
            .map(|r| {
                let d = r
                    .points
                    .iter()
                    .map(|p| {
                        i64::from(p[0] - location.x).pow(2) + i64::from(p[1] - location.y).pow(2)
                    })
                    .min()
                    .unwrap_or(i64::MAX);
                (d, r.id)
            })
            .filter(|(d, _)| *d <= 1600)
            .collect();
        candidates.sort_unstable();
        for (_, road) in candidates {
            if self.close_road(incident, road, 120).is_ok() {
                break;
            }
        }
    }
    pub(super) fn reopen_roads(&mut self) {
        let ready: Vec<_> = self
            .closures
            .iter()
            .filter(|(_, c)| c.until <= self.now)
            .map(|(id, c)| (*id, c.incident))
            .collect();
        for (road, incident) in ready {
            if let Some(n) = &mut self.network
                && let Some(r) = n.roads.iter_mut().find(|r| r.id == road)
            {
                r.closed = false;
            }
            self.closures.remove(&road);
            self.emit(EventKind::RoadReopened { incident, road });
        }
    }
}
