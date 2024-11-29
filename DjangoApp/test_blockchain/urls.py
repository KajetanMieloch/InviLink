from . import views
from django.urls import path

urlpatterns = [
    path('phantom_test/', views.phantom_test, name='phantom_test'),
    path('phantom_callback/', views.phantom_callback, name='phantom_callback'),
    path('phantom_success/', views.phantom_success, name='phantom_success'),
]