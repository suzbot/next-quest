# Test Later

Items that need manual testing but couldn't be verified immediately. Check these off as tested.

## Overlay Lane Fallback (Phase 5D Group 1, Slice 3)

Requires naturally running out of quests in Lane 1 (and optionally Lane 2) without manually burning through data to force the state. Build verified, behavior not yet observed in the wild.

- [ ] Lane 1 empty → overlay shows a Lane 2 (Adventures) quest, themed normally as a monster encounter
- [ ] While in a Lane 2 fallback state, Run / "Something Else" cycles within Lane 2 candidates only, does not hop back to Lane 1 or forward to Lane 3
- [ ] Lane 1 and Lane 2 both empty → overlay shows a Lane 3 (Royal Quests) quest
- [ ] All three lanes empty → overlay shows "All caught up. Rest well, adventurer." empty state (unchanged from prior behavior)
- [ ] After a Lane 2 fallback, if a new Lane 1 quest becomes due, the next overlay poll picks it up from Lane 1 (fallback state does not persist between polls)
- [ ] Fight / Cast Completion / Hide in the Shadows all work normally on a fallback-lane quest
