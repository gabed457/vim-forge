use vimforge::contracts::board::{ContractBoard, ContractRequirement, Contract, ContractTier, ContractStatus};
use vimforge::contracts::generation::generate_contract;
use vimforge::resources::Resource;

#[test]
fn test_contract_board_new() {
    let board = ContractBoard::new();
    assert_eq!(board.available.len(), 0);
    assert_eq!(board.active.len(), 0);
    assert_eq!(board.completed_count, 0);
    assert_eq!(board.reputation, 0);
}

#[test]
fn test_contract_requirement_fulfilled() {
    let mut req = ContractRequirement {
        resource: Resource::IronIngot,
        quantity: 10,
        delivered: 0,
    };
    assert!(!req.is_fulfilled());
    assert_eq!(req.remaining(), 10);
    req.delivered = 10;
    assert!(req.is_fulfilled());
    assert_eq!(req.remaining(), 0);
}

#[test]
fn test_contract_deliver() {
    let mut contract = Contract {
        id: 1,
        name: "Test Contract".to_string(),
        tier: ContractTier::Starter,
        requirements: vec![ContractRequirement {
            resource: Resource::IronIngot,
            quantity: 10,
            delivered: 0,
        }],
        reward: 1000,
        bonus_reward: 200,
        penalty: -500,
        deadline: 1000,
        issued_at: 0,
        status: ContractStatus::Active,
        reputation_reward: 10,
        reputation_penalty: -5,
    };
    let delivered = contract.deliver(Resource::IronIngot, 5);
    assert_eq!(delivered, 5);
    assert_eq!(contract.requirements[0].delivered, 5);
    assert!(!contract.is_complete());

    let delivered = contract.deliver(Resource::IronIngot, 10);
    assert_eq!(delivered, 5); // only 5 remaining
    assert!(contract.is_complete());
}

#[test]
fn test_contract_wrong_resource() {
    let mut contract = Contract {
        id: 1,
        name: "Test".to_string(),
        tier: ContractTier::Starter,
        requirements: vec![ContractRequirement {
            resource: Resource::IronIngot,
            quantity: 10,
            delivered: 0,
        }],
        reward: 1000,
        bonus_reward: 0,
        penalty: 0,
        deadline: 1000,
        issued_at: 0,
        status: ContractStatus::Active,
        reputation_reward: 0,
        reputation_penalty: 0,
    };
    let delivered = contract.deliver(Resource::CopperIngot, 5);
    assert_eq!(delivered, 0);
}

#[test]
fn test_contract_tier_names() {
    assert_eq!(ContractTier::Starter.name(), "Starter");
    assert_eq!(ContractTier::Legendary.name(), "Legendary");
}

#[test]
fn test_generate_contract() {
    let unlocked = vec![Resource::IronIngot, Resource::CopperIngot];
    let contract = generate_contract(ContractTier::Starter, 1, &unlocked, 0, 1);
    assert_eq!(contract.id, 1);
    assert!(!contract.requirements.is_empty());
    assert!(contract.reward > 0);
    assert!(contract.deadline > 0);
}

#[test]
fn test_contract_accept() {
    let mut board = ContractBoard::new();
    let contract = Contract {
        id: 1,
        name: "Test".to_string(),
        tier: ContractTier::Starter,
        requirements: vec![],
        reward: 100,
        bonus_reward: 0,
        penalty: 0,
        deadline: 1000,
        issued_at: 0,
        status: ContractStatus::Available,
        reputation_reward: 0,
        reputation_penalty: 0,
    };
    board.add_available(contract);
    assert_eq!(board.available.len(), 1);
    assert!(board.accept(1, 100));
    assert_eq!(board.available.len(), 0);
    assert_eq!(board.active.len(), 1);
}
