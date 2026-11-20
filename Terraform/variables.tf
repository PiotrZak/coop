variable "project_id" {
  description = "Google Cloud Project ID (your project — not Lindle's)"
  type        = string
}

variable "region" {
  description = "Google Cloud region"
  type        = string
  default     = "europe-central2"
}

variable "service_name" {
  description = "Cloud Run + Artifact Registry repository id"
  type        = string
  default     = "coop-a2a"
}

variable "image_tag" {
  description = "Container image tag (e.g. prod-<git-sha>). Placeholder until first CI push."
  type        = string
  default     = "placeholder"
}

variable "github_sa_email" {
  description = "Deployer service account email used by GitHub Actions (create SA out-of-band; do not commit keys)"
  type        = string
}

variable "gar_location" {
  description = "Artifact Registry location"
  type        = string
  default     = "europe-central2"
}

variable "allow_unauthenticated" {
  description = "Whether Cloud Run allows unauthenticated invocations (demo Agent Card)"
  type        = bool
  default     = true
}
