import type { Meta, StoryObj } from '@storybook/react';
import { KavanaCompanion } from './KavanaCompanion';

const meta: Meta<typeof KavanaCompanion> = { title: 'Experiences/Kavana Companion', component: KavanaCompanion, tags: ['autodocs'], parameters: { layout: 'centered', docs: { description: { component: 'Kavana is Caro’s animated project guide. She uses the complete Codex v2 sprite atlas to roam, explain development status, and help visitors adopt or hatch a pet.' } } } };
export default meta;
type Story = StoryObj<typeof KavanaCompanion>;
export const Introduction: Story = { args: { embedded: true, initiallyOpen: true } };
export const Roaming: Story = { args: { embedded: false, initiallyOpen: false }, parameters: { layout: 'fullscreen' } };
