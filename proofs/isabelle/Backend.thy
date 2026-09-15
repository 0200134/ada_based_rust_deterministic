theory Backend
  imports Main
begin

datatype expr = Literal int | Add expr expr | Sub expr expr | Mul expr expr

primrec eval :: "expr => int" where
  "eval (Literal value) = value"
| "eval (Add left right) = eval left + eval right"
| "eval (Sub left right) = eval left - eval right"
| "eval (Mul left right) = eval left * eval right"

primrec fold :: "expr => expr" where
  "fold (Literal value) = Literal value"
| "fold (Add left right) = Literal (eval left + eval right)"
| "fold (Sub left right) = Literal (eval left - eval right)"
| "fold (Mul left right) = Literal (eval left * eval right)"

lemma fold_preserves_evaluation:
  "eval (fold expression) = eval expression"
  by (induction expression) simp_all

lemma nat_add_is_deterministic:
  "(left::nat) + right = left + right"
  by sledgehammer

lemma nat_add_associative:
  "((left::nat) + right) + third = left + (right + third)"
  by sledgehammer

end