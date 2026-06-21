<!-- Skeleton with the SAME shape as SubscriptionRow (chip · name/cat · lifebar ·
     amount · menu). Standard: never "Loading…", always the component's skeleton. -->
<div class="row">
	<div class="left">
		<span class="chip skeleton"></span>
		<div class="names">
			<span class="skeleton l1"></span>
			<span class="skeleton l2"></span>
		</div>
	</div>
	<div class="mid">
		<span class="skeleton bar"></span>
		<span class="skeleton under"></span>
	</div>
	<div class="right">
		<span class="skeleton amount"></span>
		<span class="skeleton cyc"></span>
	</div>
	<span class="skeleton menu"></span>
</div>

<style>
	.row {
		display: grid;
		grid-template-columns: minmax(160px, 1.05fr) 1.9fr minmax(96px, 0.6fr) 34px;
		align-items: center;
		gap: 1.25rem;
		padding: 1rem 1.5rem 1rem 2.2rem;
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-lg, 16px);
	}
	.left {
		display: flex;
		align-items: center;
		gap: 1rem;
	}
	.chip {
		width: 38px;
		height: 38px;
		border-radius: 11px;
		flex: none;
	}
	.names {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.l1 {
		width: 110px;
		height: 12px;
	}
	.l2 {
		width: 64px;
		height: 9px;
	}
	.mid {
		display: flex;
		flex-direction: column;
		gap: 0.55rem;
	}
	.bar {
		height: 12px;
		border-radius: 999px;
	}
	.under {
		width: 130px;
		height: 9px;
	}
	.right {
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		gap: 6px;
	}
	.amount {
		width: 70px;
		height: 13px;
	}
	.cyc {
		width: 46px;
		height: 9px;
	}
	.menu {
		width: 24px;
		height: 24px;
		border-radius: 8px;
		justify-self: center;
	}
</style>
