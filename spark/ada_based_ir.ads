package Ada_Based_IR
  with SPARK_Mode
is
   Max_Instructions : constant := 256;
   subtype Value_Id is Natural range 0 .. Max_Instructions - 1;

   type Opcode is (Constant_Value, Add, Subtract, Multiply, Divide, Remainder,
                   Negate, Equal, Less_Than, Select_Value, Return_Value);

   type Instruction is record
      Operation : Opcode := Constant_Value;
      Result : Value_Id := 0;
      Left : Value_Id := 0;
      Right : Value_Id := 0;
      Immediate : Integer := 0;
   end record;

   type Instruction_Array is array (Value_Id range <>) of Instruction;

   type Program is record
      Count : Natural range 0 .. Max_Instructions := 0;
      Code : Instruction_Array (Value_Id'First .. Value_Id'Last);
   end record;

   function Opcode_Text (Operation : Opcode) return String;

   procedure Append
     (State : in out Program;
      Item : in Instruction)
   with
      Pre  => State.Count < Max_Instructions,
      Post => State.Count = State.Count'Old + 1
              and then State.Code (State.Count - 1) = Item;

   function Verify (State : Program) return Boolean
     with
       Post => Verify'Result =
         (State.Count <= Max_Instructions);
end Ada_Based_IR;
