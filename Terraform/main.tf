terraform {
  required_version = ">= 1.5.0"
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 5.0"
    }
  }

  # Optional remote state — set via backend.hcl / -backend-config (no secrets in git).
  # backend "gcs" {}
}

provider "google" {
  project = var.project_id
  region  = var.region
}
