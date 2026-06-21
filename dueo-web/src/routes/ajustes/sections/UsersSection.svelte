<script lang="ts">
	import { onMount } from 'svelte';
	import { fade, slide } from 'svelte/transition';
	import { Plus, Trash2 } from '@lucide/svelte';
	import { getUsers, createUser, deleteUser, type AdminUser } from '$lib/api';
	import { i18n } from '$lib/i18n.svelte';
	import { busy } from './busy.svelte';

	// meId marks the current admin so they can't delete themselves.
	let { meId }: { meId: number | null } = $props();

	let users = $state<AdminUser[]>([]);
	let loading = $state(true);
	let newUsername = $state('');
	let newPassword = $state('');
	let newRole = $state('member');
	let err = $state('');
	let confirmDel = $state<number | null>(null);
	const creating = busy();

	onMount(async () => {
		const res = await getUsers();
		if (res.ok && res.data) users = res.data;
		loading = false;
	});

	function addUser() {
		err = '';
		if (!newUsername.trim()) return (err = i18n.t('set.usersErrName'));
		if (newPassword.length < 8) return (err = i18n.t('set.usersErrPass'));
		creating.run(async () => {
			const res = await createUser({
				username: newUsername.trim(),
				password: newPassword,
				role: newRole
			});
			if (res.ok && res.data) {
				users = [...users, res.data];
				newUsername = '';
				newPassword = '';
				newRole = 'member';
			} else {
				err = i18n.t('set.usersErrCreate');
			}
		});
	}

	async function removeUser(u: AdminUser) {
		const res = await deleteUser(u.id);
		if (res.ok || res.status === 404) {
			users = users.filter((x) => x.id !== u.id);
		} else {
			err = i18n.t('set.usersErrDelete');
		}
		confirmDel = null;
	}
</script>

{#if loading}
	<div class="userlist">
		{#each Array(2) as _, i (i)}
			<div class="urow">
				<div class="skeleton" style="width:40%;height:1rem"></div>
				<div class="skeleton" style="width:64px;height:1rem"></div>
			</div>
		{/each}
	</div>
{:else}
	<div class="userlist">
		{#each users as u (u.id)}
			<div class="urow" transition:slide={{ duration: 160 }}>
				<div class="uinfo">
					<span class="uname">{u.username}</span>
					<span class="urole" class:admin={u.role === 'admin'}
						>{u.role === 'admin' ? i18n.t('set.roleAdmin') : i18n.t('set.roleMember')}</span
					>
					{#if u.id === meId}<span class="ume">{i18n.t('set.you')}</span>{/if}
				</div>
				{#if u.id !== meId}
					{#if confirmDel === u.id}
						<div class="confirm" transition:fade={{ duration: 120 }}>
							<button class="danger" onclick={() => removeUser(u)}>{i18n.t('common.delete')}</button
							>
							<button class="ghost sm" onclick={() => (confirmDel = null)}
								>{i18n.t('common.cancel')}</button
							>
						</div>
					{:else}
						<button
							class="icon-btn"
							aria-label={i18n.t('set.deleteUser')}
							onclick={() => (confirmDel = u.id)}
						>
							<Trash2 size={15} />
						</button>
					{/if}
				{/if}
			</div>
		{/each}
	</div>
{/if}

<form class="newuser" onsubmit={(e) => (e.preventDefault(), addUser())}>
	<input bind:value={newUsername} placeholder={i18n.t('set.userPlaceholder')} autocomplete="off" />
	<input
		bind:value={newPassword}
		type="password"
		placeholder={i18n.t('set.passPlaceholder')}
		autocomplete="new-password"
	/>
	<select bind:value={newRole}>
		<option value="member">{i18n.t('set.roleMember')}</option>
		<option value="admin">{i18n.t('set.roleAdmin')}</option>
	</select>
	<button type="submit" class="ghost" disabled={creating.on}>
		<Plus size={15} />
		{creating.on ? i18n.t('set.creating') : i18n.t('common.create')}
	</button>
</form>
{#if err}<p transition:slide={{ duration: 180 }} class="err">{err}</p>{/if}

<style>
	.userlist {
		display: flex;
		flex-direction: column;
		gap: 0.4rem;
	}
	.urow {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.6rem;
		padding: 0.55rem 0.7rem;
		border-radius: 10px;
		background: var(--surface-2);
		border: 1px solid var(--border);
	}
	.uinfo {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		min-width: 0;
	}
	.uname {
		font-size: 0.9rem;
		font-weight: 550;
		color: var(--text);
	}
	.urole {
		font-size: 0.7rem;
		padding: 2px 8px;
		border-radius: 999px;
		color: var(--text-2);
		background: color-mix(in srgb, var(--text) 8%, transparent);
		text-transform: capitalize;
	}
	.urole.admin {
		color: var(--brand);
		background: color-mix(in srgb, var(--brand) 14%, transparent);
	}
	.ume {
		font-size: 0.7rem;
		color: var(--text-muted);
	}
	.confirm {
		display: flex;
		gap: 0.4rem;
	}
	.icon-btn {
		display: grid;
		place-items: center;
		width: 30px;
		height: 30px;
		border: 1px solid var(--border);
		border-radius: 8px;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
	}
	.icon-btn:hover {
		color: var(--danger);
		border-color: color-mix(in srgb, var(--danger) 40%, transparent);
		background: color-mix(in srgb, var(--danger) 10%, transparent);
	}
	.danger {
		display: inline-flex;
		align-items: center;
		height: 32px;
		padding: 0 0.8rem;
		border: none;
		border-radius: 9px;
		font-size: 0.82rem;
		font-weight: 600;
		color: white;
		background: var(--danger);
		cursor: pointer;
	}
	.ghost.sm {
		height: 32px;
		padding: 0 0.7rem;
		font-size: 0.82rem;
	}
	.newuser {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
	}
	.newuser input {
		flex: 1;
		min-width: 130px;
	}
	/* "Create" jumps to its own full-width row as a clear CTA. */
	.newuser button {
		flex: 1 0 100%;
		justify-content: center;
	}
	.err {
		margin: 0;
		color: var(--danger);
		font-size: 0.82rem;
	}
</style>
