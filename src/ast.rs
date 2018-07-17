use pest::iterators::Pair;

pub enum Value {
    True,
    False,
    Zero,
}

pub enum Operator {
    Succ,
    Pred,
}

pub enum Type {
    Bool,
    Nat,
}

pub enum ASTNode {
    AbstractionNode{
        ident: Box<ASTNode>,
        data_type: Type,
        body: Box<ASTNode>,        
    },
    ApplicationNode{
        left: Box<ASTNode>, 
        right: Box<ASTNode>,
    },
    IdentifierNode{
        name: String
    },
    ConditionNode{
        clause: Box<ASTNode>,
        then_arm: Box<ASTNode>, 
        else_arm: Box<ASTNode>
    },
    ArithmeticNode{
        op: Operator, 
        expr: Box<ASTNode>
    },
    IsZeroNode{
        expr: Box<ASTNode>
    },
    ValueNode{
        value: Value
    },
}



pub fn build_ast() {}
