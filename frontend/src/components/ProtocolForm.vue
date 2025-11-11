<script setup lang="ts">
interface ProtocolFormModel {
  name: string;
  peptideName: string;
  notes: string;
  targetConcentration: string | number;
}

const props = defineProps<{
  form: ProtocolFormModel;
  saving: boolean;
}>();

const emit = defineEmits<{
  submit: [];
  'update:name': [value: string];
  'update:peptideName': [value: string];
  'update:notes': [value: string];
  'update:targetConcentration': [value: string | number];
}>();

function handleSubmit() {
  emit("submit");
}
</script>

<template>
  <article class="panel">
    <div class="panel-header">
      <h2>➕ Add New Peptide Plan</h2>
    </div>
    <form class="form-stack" @submit.prevent="handleSubmit">
      <label>
        Plan Name
        <input :value="props.form.name" @input="emit('update:name', ($event.target as HTMLInputElement).value)" type="text" placeholder="e.g., Morning Routine, Recovery Plan" />
      </label>
      <label>
        Which Peptide?
        <input
          :value="props.form.peptideName"
          @input="emit('update:peptideName', ($event.target as HTMLInputElement).value)"
          type="text"
          placeholder="e.g., BPC-157, TB-500"
        />
      </label>
      <label>
        Strength (optional)
        <input
          :value="props.form.targetConcentration"
          @input="emit('update:targetConcentration', ($event.target as HTMLInputElement).value)"
          type="number"
          min="0"
          step="0.01"
          placeholder="mg/mL"
        />
      </label>
      <label>
        Notes (optional)
        <textarea
          :value="props.form.notes"
          @input="emit('update:notes', ($event.target as HTMLTextAreaElement).value)"
          rows="3"
          placeholder="Where you bought it, how to mix it, storage instructions, etc."
        />
      </label>
      <button class="primary" type="submit" :disabled="props.saving">
        {{ props.saving ? "Saving..." : "💾 Save Plan" }}
      </button>
    </form>
  </article>
</template>
