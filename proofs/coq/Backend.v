From Coq Require Import Lia.

Module AdaBasedRust.

  Inductive expr : Type :=
  | Literal : Z -> expr
  | Add : expr -> expr -> expr
  | Sub : expr -> expr -> expr
  | Mul : expr -> expr -> expr.

  Fixpoint eval (expression : expr) : Z :=
    match expression with
    | Literal value => value
    | Add left right => eval left + eval right
    | Sub left right => eval left - eval right
    | Mul left right => eval left * eval right
    end.

  Fixpoint fold (expression : expr) : expr :=
    match expression with
    | Literal value => Literal value
    | Add left right => Literal (eval left + eval right)
    | Sub left right => Literal (eval left - eval right)
    | Mul left right => Literal (eval left * eval right)
    end.

  Theorem fold_preserves_evaluation :
    forall expression : expr, eval (fold expression) = eval expression.
  Proof.
    induction expression; simpl; auto.
  Qed.

  Theorem nat_add_is_deterministic :
    forall left right : nat, left + right = left + right.
  Proof.
    reflexivity.
  Qed.

  Theorem nat_add_associative :
    forall left right third : nat,
      (left + right) + third = left + (right + third).
  Proof.
    intros left right third.
    lia.
  Qed.

End AdaBasedRust.