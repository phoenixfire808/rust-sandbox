param([string]$Sheets = (Join-Path $PSScriptRoot '..\sheets'))
$ErrorActionPreference = 'Stop'
$path = Join-Path $Sheets 'source_npcs.csv'
if (Test-Path $path) { throw 'NPC sheet already exists. Edit authored CSV instead of overwriting it.' }
# One-time coverage bootstrap. CSV is authoritative after creation.
$enabled = @{
 npc_npc_zombie = @('melee','models/zombie/classic.mdl','zombie',100,0.8,1.1,10,1.2,1.37,0.3)
 npc_npc_zombie_torso = @('melee','models/zombie/classic_torso.mdl','zombie',50,0.35,0.8,10,1.4,0.65,0.25)
 npc_npc_combine_s = @('ranged','models/combine_soldier.mdl','combine',50,2.2,12,4,0.4,1.37,0.3)
 npc_combineelite = @('ranged','models/combine_super_soldier.mdl','combine',70,2.2,16,8,0.6,1.37,0.3)
 npc_combineprison = @('ranged','models/combine_soldier_prisonguard.mdl','combine',50,2.2,12,4,0.4,1.37,0.3)
 npc_npc_odessa = @('passive','models/odessa.mdl','resistance',100,1.2,1,0,1,1.37,0.3)
}
$rows = foreach ($entry in Import-Csv (Join-Path $Sheets 'spawn_reference.csv') | Where-Object kind -eq npc) {
 $v = $enabled[$entry.id]
 if (!$v) { $v = @('disabled',$entry.model,'neutral',0,0,0,0,1,1,0.3) }
 $clips = switch ($entry.id) {
  'npc_npc_zombie' { '@Idle01'; 'a_walk1'; '@attackA' }
  'npc_npc_zombie_torso' { '@Idle01'; '@crawl'; '@attack' }
  'npc_npc_odessa' { 'models/humans/male_shared.mdl::@idle_subtle'; 'models/humans/male_shared.mdl::a_WalkN'; 'models/humans/male_shared.mdl::@idle_subtle' }
  default { if ($v[0] -eq 'ranged') { 'models/combine_soldier_anims.mdl::@CombatIdle1'; 'models/combine_soldier_anims.mdl::a_WalkN'; 'models/combine_soldier_anims.mdl::@shootAR2s' } else { ''; ''; '' } }
 }
 [pscustomobject][ordered]@{
  id=$entry.id; kind=$v[0]; model=$v[1]; faction=$v[2]; health=$v[3]; speed=$v[4]; range=$v[5]; damage=$v[6]; interval=$v[7]; sight=30; height=$v[8]; radius=$v[9]; forward_yaw=0
  idle=$clips[0]; walk=$clips[1]; attack=$clips[2]
  scope=$(if ($v[0] -eq 'disabled') {'Reference-only; no substitute actor'} else {'Typed ground actor with candidate original clips and explicitly prototype combat tuning'})
  remaining='Native schedules navigation hitgroups equipment sounds ragdolls and class-specific actions; gameplay acceptance'
 }
}
$rows | Export-Csv $path -NoTypeInformation -Encoding UTF8
Write-Output "Created $($rows.Count) explicit NPC coverage rows. No gameplay acceptance."
