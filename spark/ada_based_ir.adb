package body Ada_Based_IR
  with SPARK_Mode
is
   function Opcode_Text (Operation : Opcode) return String is
   begin
      case Operation is
         when Constant_Value => return "const";
         when Add => return "add";
         when Subtract => return "sub";
         when Multiply => return "mul";
         when Divide => return "sdiv";
         when Remainder => return "srem";
         when Negate => return "neg";
         when Equal => return "icmp_eq";
         when Less_Than => return "icmp_slt";
         when Select_Value => return "select";
         when Return_Value => return "ret";
      end case;
   end Opcode_Text;

   procedure Append
     (State : in out Program;
      Item : in Instruction) is
   begin
      State.Code (State.Count) := Item;
      State.Count := State.Count + 1;
   end Append;

   function Verify (State : Program) return Boolean is
   begin
      return State.Count <= Max_Instructions;
   end Verify;
end Ada_Based_IR;
