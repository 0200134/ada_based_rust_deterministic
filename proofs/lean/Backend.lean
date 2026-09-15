namespace AdaBasedRust

inductive Expr where
  | literal : Int -> Expr
  | add : Expr -> Expr -> Expr
  | sub : Expr -> Expr -> Expr
  | mul : Expr -> Expr -> Expr

def eval : Expr -> Int
  | .literal value => value
  | .add left right => eval left + eval right
  | .sub left right => eval left - eval right
  | .mul left right => eval left * eval right

def fold : Expr -> Expr
  | .literal value => .literal value
  | .add left right => .literal (eval left + eval right)
  | .sub left right => .literal (eval left - eval right)
  | .mul left right => .literal (eval left * eval right)

theorem foldPreservesEvaluation (expression : Expr) :
    eval (fold expression) = eval expression := by
  induction expression with
  | literal value => rfl
  | add left right => simp [fold, eval]
  | sub left right => simp [fold, eval]
  | mul left right => simp [fold, eval]

theorem natAddIsDeterministic (left right : Nat) :
    left + right = left + right := by
  rfl

theorem natAddAssociative (left right third : Nat) :
    (left + right) + third = left + (right + third) := by
  exact Nat.add_assoc left right third

end AdaBasedRust