package Ada_Based_Runtime
  with SPARK_Mode
is
   subtype Error_Code is Natural range 0 .. 65_535;

   type Lifecycle is (Running, Stopped);

   type Runtime_State is record
      Phase : Lifecycle := Running;
      Next_Sequence : Positive := 1;
   end record;

   procedure Record_Error
     (State : in out Runtime_State;
      Code : in Error_Code)
   with
      Pre  => State.Phase = Running
              and then State.Next_Sequence < Positive'Last,
         Post => State.Phase = Running
            and then State.Next_Sequence = State.Next_Sequence'Old + 1;

   procedure Shutdown (State : in out Runtime_State)
   with
      Pre  => State.Phase = Running,
      Post => State.Phase = Stopped;
end Ada_Based_Runtime;