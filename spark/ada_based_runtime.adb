package body Ada_Based_Runtime
  with SPARK_Mode
is
   procedure Record_Error
     (State : in out Runtime_State;
      Code : in Error_Code) is
   begin
      pragma Unreferenced (Code);
      State.Next_Sequence := State.Next_Sequence + 1;
   end Record_Error;

   procedure Shutdown (State : in out Runtime_State) is
   begin
      State.Phase := Stopped;
   end Shutdown;
end Ada_Based_Runtime;