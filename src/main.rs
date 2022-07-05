#![allow(unused, unused_variables, unused_imports)]

use itertools::Itertools;
use proptest::prelude::*;
use proptest_derive::Arbitrary;
use std::collections::{HashMap, HashSet};

// Maps rider id to sequence of order ids
type Plan = HashMap<u32, Vec<u64>>;

#[derive(Arbitrary, Clone, Debug)]
struct Rider {
    id: u32,
}

#[derive(Arbitrary, Clone, Debug)]
struct Order {
    id: u64,
}

fn compute_plan(riders: &[Rider], orders: &[Order]) -> Plan {
    let mut plan = Plan::default();
    /* 1st implementation */
    /*
    let mut next_order_idx = 0;
    for rider in riders {
        plan.insert(rider.id, vec![orders[next_order_idx].id]);
        next_order_idx += 1;
    }
    */

    /* end */
    /* 2nd implementation */
    /*
    let mut next_order_idx = 0;
    for rider in riders {
        plan.insert(rider.id, vec![orders[next_order_idx].id]);
        next_order_idx += 1;
    }
    for order in &orders[next_order_idx..] {
        plan.entry(riders[0].id).or_default().push(order.id);
    }
    */
    /* end */

    /* 3rd implementation */
    let mut next_order_idx = 0;
    loop {
        for rider_idx in 0..riders.len() {
            if next_order_idx >= orders.len() {
                return plan;
            }
            plan.entry(riders[rider_idx].id)
                .or_default()
                .push(orders[next_order_idx].id);
            next_order_idx += 1;
        }
    }
    /* end */
    plan
}

#[derive(Arbitrary, Clone, Debug)]
enum TestEvent {
    RiderRejected {
        which_rider: usize,
        which_order: usize,
    },
    OrderCanceled {
        which_order: usize,
    },
}

impl TestEvent {
    fn into_event(self, plan: &Plan) -> Event {
        match self {
            Self::RiderRejected {
                which_rider,
                which_order,
            } => {
                let all_sorted_riders: Vec<u32> = plan.keys().cloned().sorted().dedup().collect();
                let len = all_sorted_riders.len();
                let rider_id = all_sorted_riders[which_rider % len];
                let orders_of_rider = &plan[&rider_id];
                let order_id = orders_of_rider[which_order % orders_of_rider.len()];
                Event::RiderRejected { rider_id, order_id }
            }
            Self::OrderCanceled { which_order } => {
                let all_sorted_orders: Vec<u64> =
                    plan.values().flatten().cloned().sorted().dedup().collect();
                let len = all_sorted_orders.len();
                Event::OrderCanceled {
                    order_id: all_sorted_orders[which_order as usize % len],
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Event {
    RiderRejected { rider_id: u32, order_id: u64 },
    OrderCanceled { order_id: u64 },
}

fn process_event(mut plan: Plan, event: Event) -> Plan {
    match event {
        Event::RiderRejected { rider_id, order_id } => {
            // Move order to other rider
            /* 1st implementation */
            /*
            plan.get_mut(&rider_id)
                .map(|orders| orders.retain(|v| *v != order_id));
            if let Some((_, orders)) = plan.iter_mut().find(|(id, _)| **id != rider_id) {
                orders.push(order_id);
            }
            */
            /* end */
            /* 2nd implementation */
            if let Some(orders) = plan.get_mut(&rider_id) {
                if let Some(idx) = orders.iter().position(|v| *v == order_id) {
                    orders.remove(idx);
                    if let Some((_, orders)) = plan.iter_mut().find(|(id, _)| **id != rider_id)
                    {
                        orders.push(order_id);
                    }
                }
            }
            /* end */
        }
        Event::OrderCanceled { order_id } => {
            // Remove order from plan
            plan.values_mut()
                .for_each(|orders| orders.retain(|v| *v != order_id));
        }
    }
    plan
}








proptest! {
    #[test]
    fn all_riders_get_orders(riders: Vec<Rider>, orders: Vec<Order>) {
        prop_assume!(!riders.is_empty());
        prop_assume!(riders.len() <= orders.len());
        prop_assume!(riders.iter().map(|r| r.id).all_unique());
        prop_assume!(orders.iter().map(|o| o.id).all_unique());
        println!("{} {}", riders.len(), orders.len());

        let plan = compute_plan(&riders, &orders);
        for rider in riders {
            assert!(plan.contains_key(&rider.id));
            assert!(!plan[&rider.id].is_empty());
        }
    }










    #[test]
    fn all_orders_are_assigned(riders: Vec<Rider>, orders: Vec<Order>) {
        prop_assume!(!riders.is_empty());
        prop_assume!(riders.len() <= orders.len());
        prop_assume!(riders.iter().map(|r| r.id).all_unique());
        prop_assume!(orders.iter().map(|o| o.id).all_unique());

        println!("{} {}", riders.len(), orders.len());

        let plan = compute_plan(&riders, &orders);
        for order in orders {
            assert!(plan.values().any(|v| v.contains(&order.id)));
        }
    }










    #[test]
    fn orders_are_assigned_in_an_even_way(riders: Vec<Rider>, orders: Vec<Order>) {
        prop_assume!(!riders.is_empty());
        prop_assume!(riders.len() <= orders.len());
        prop_assume!(riders.iter().map(|r| r.id).all_unique());
        prop_assume!(orders.iter().map(|o| o.id).all_unique());

        let plan = compute_plan(&riders, &orders);
        let (min_orders, max_orders) = plan.values().map(|orders| orders.len()).minmax().into_option().unwrap();
        assert!(1 >= max_orders-min_orders, "min: {}, max: {}", min_orders, max_orders);
    }









    #[test]
    fn events_over_time(starting_plan: Plan, test_events: Vec<TestEvent>) {
        prop_assume!(starting_plan.len() > 1);
        prop_assume!(starting_plan.values().all(|orders| !orders.is_empty()));
        prop_assume!(starting_plan.values().flatten().all_unique());

        let events : Vec<Event> = test_events.into_iter().map(|test_event| test_event.into_event(&starting_plan)).collect();
        let canceled_orders : HashSet<_> = events.iter()
            .flat_map(|e| if let Event::OrderCanceled{order_id} = e { Some(*order_id) } else { None })
            .collect();
        println!("total starting orders {}, events {}", starting_plan.values().map(|v| v.len()).sum::<usize>(), canceled_orders.len());
        let mut current_plan = starting_plan.clone();
        for event in events {
            let orders_before : HashSet<_> = current_plan.values().flatten().cloned().collect();
            current_plan = process_event(current_plan, event.clone());
            if let Event::RiderRejected{rider_id,order_id} = event {
                let orders_after : HashSet<_> = current_plan.values().flatten().cloned().collect();
                assert_eq!(orders_before, orders_after);
                assert!(!current_plan[&rider_id].contains(&order_id));
            }
        }
        let remaining_orders : HashSet<u64> = current_plan.values().flatten().cloned().collect();
        assert!(canceled_orders.iter().all(|canceled| !remaining_orders.contains(canceled)));
        assert_eq!(starting_plan.values().flatten().collect::<HashSet<_>>(), 
            canceled_orders.iter().chain(remaining_orders.iter()).collect());
    }
}

fn main() {}
